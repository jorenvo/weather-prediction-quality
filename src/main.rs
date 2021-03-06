#![warn(clippy::all)]
use chrono::{Datelike, Local, NaiveDate};
use clap::{App, Arg, ArgMatches};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::{Read, Write};

const DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Serialize, Deserialize, Debug)]
struct TempPrediction {
    made_at: String,
    low: String,
    high: String,
}

struct Prediction {
    for_date: String,
    temp: TempPrediction,
}

#[derive(Serialize, Deserialize, Debug)]
struct SavedPredictions {
    predictions: BTreeMap<String, Vec<TempPrediction>>,
}

fn query_weather_com() -> reqwest::blocking::Response {
    println!("Querying...");
    let url = "https://weather.com/weather/tenday/l/\
San+Francisco+CA?canonicalCityId=dfdaba8cbe3a4d12a8796e1f7b1ccc7174b4b0a2d5ddb1c8566ae9f154fa638c";
    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send().unwrap();
    println!("Got {}", response.status());
    response
}

fn select_first_inner(element: scraper::element_ref::ElementRef, selector: &str) -> String {
    let selector = scraper::Selector::parse(selector).unwrap();
    String::from(
        element
            .select(&selector)
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap(),
    )
}

fn scrape_info(html: &str) -> Vec<Prediction> {
    let now = Local::now().naive_local();
    let fragment = scraper::Html::parse_fragment(html);
    let selector = scraper::Selector::parse("[data-testid=\"DetailsSummary\"]").unwrap();
    let mut predictions = vec![];

    for prediction in fragment.select(&selector) {
        let day = select_first_inner(prediction, "[data-testid=\"daypartName\"");
        let date = match day.split_whitespace().nth(1) {
            Some(day) => {
                // Fri 05
                let day = day.parse::<u32>().unwrap();
                let mut month = now.month();
                let mut year = now.year();

                if day < now.day() {
                    month += 1;
                    if month > 12 {
                        month = 1;
                        year += 1;
                    }
                }

                NaiveDate::from_ymd(year, month, day)
            }
            None => now.date(), // Tonight, Today
        };

        let date = date.format(DATE_FORMAT).to_string();
        let low = select_first_inner(
            prediction,
            "[data-testid=\"lowTempValue\"] > [data-testid=\"TemperatureValue\"]",
        )
        .replace("°", "");
        let high =
            select_first_inner(prediction, "[data-testid=\"TemperatureValue\"]").replace("°", "");
        let made_at = now.format(DATE_FORMAT).to_string();
        let temp = TempPrediction { made_at, low, high };
        predictions.push(Prediction {
            for_date: date,
            temp,
        });
    }

    predictions
}

fn read_data_store(data_store_path: &str) -> String {
    let mut data_store_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(data_store_path)
        .unwrap();
    let mut data_store = String::new();
    data_store_file.read_to_string(&mut data_store).unwrap();

    if data_store.is_empty() {
        data_store = serde_json::to_string(&SavedPredictions {
            predictions: BTreeMap::new(),
        })
        .unwrap();
    }

    data_store
}

fn write_data_store(data_store_path: &str, serialized: String) {
    let mut data_store_file = OpenOptions::new()
        .write(true)
        .open(data_store_path)
        .unwrap();
    data_store_file.write_all(serialized.as_bytes()).unwrap();
}

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("weather.com prediction accuracy analysis")
        .version("1.0")
        .author("Joren Van Onder <joren@jvo.sh>")
        .about("Scrapes weather.com and produces CSVs that can be plotted.")
        .arg(
            Arg::with_name("data store")
                .short("d")
                .long("--data-store")
                .value_name("FILE")
                .help("Path to the data store")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("csv directory")
                .short("c")
                .long("--csv-directory")
                .value_name("DIRECTORY")
                .help("Directory where all CSVs will be stored")
                .required(true)
                .takes_value(true),
        )
        .get_matches()
}

fn main() {
    let args = parse_args();
    let data_store_path = args.value_of("data store").unwrap();
    let csv_directory = args.value_of("csv directory").unwrap();
    let data_store = read_data_store(data_store_path);

    let response = query_weather_com();
    let predictions = scrape_info(&response.text().unwrap());

    println!("Disk data store: {}", data_store);
    let mut saved_predictions: SavedPredictions = serde_json::from_str(&data_store).unwrap();
    for prediction in predictions {
        let prev_values = saved_predictions
            .predictions
            .entry(prediction.for_date)
            .or_insert_with(Vec::new);

        if prev_values.is_empty() || prev_values.last().unwrap().made_at != prediction.temp.made_at
        {
            prev_values.push(prediction.temp);
        }
    }

    for (date, temperatures) in &saved_predictions.predictions {
        println!("{}: {:?}", date, temperatures);
    }

    write_data_store(
        data_store_path,
        serde_json::to_string(&saved_predictions).unwrap(),
    );

    for (for_date, predictions) in saved_predictions.predictions {
        let mut content = String::new();
        content.push_str(&format!("at date\tlow\thigh\n"));
        for prediction in predictions {
            content.push_str(&format!(
                "{}\t{}\t{}\n",
                prediction.made_at, prediction.low, prediction.high
            ));
        }

        let mut csv_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(format!("{}/{}.csv", csv_directory, for_date))
            .unwrap();
        csv_file.write_all(content.as_bytes()).unwrap();
    }
}
