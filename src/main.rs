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
        let date = if day == "Tonight" {
            now.date()
        } else {
            // Fri 05
            let day = day
                .split_whitespace()
                .nth(1)
                .unwrap()
                .parse::<u32>()
                .unwrap();
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
            .or_insert(vec![]);

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

    // let csv_file = File::open(get_first_arg()).expect("couldn't open file");
    // let mut wtr = WriterBuilder::new().delimiter(b'\t').from_writer(vec![]);
    // let mut rdr = ReaderBuilder::new().delimiter(b'\t').from_reader(csv_file);
    //
    // let existing_headers: Vec<String> = rdr
    //     .headers()
    //     .expect("couldn't read headers")
    //     .iter()
    //     .map(String::from)
    //     .collect();
    //
    // let mut start_column = 0;
    // for (i, header) in existing_headers.iter().enumerate() {
    //     if header == "prediction date" {
    //         continue;
    //     }
    //
    //     let column_date = NaiveDate::parse_from_str(header, DATE_FORMAT).unwrap();
    //     if column_date == predictions[0].date {
    //         start_column = i;
    //     }
    // }
    //
    // for (record_i, record) in rdr.records().enumerate() {
    //     print!("{} :: ", record_i);
    //     for (field_i, field) in record.unwrap().iter().enumerate() {
    //         if field_i == 0 {
    //             print!("prediction date: {}", field);
    //         } else {
    //             print!(
    //                 "\t{}: {}",
    //                 NaiveDate::parse_from_str(&existing_headers[field_i], DATE_FORMAT)
    //                     .unwrap_or_else(|_| panic!("invalid date {}", field)),
    //                 &field,
    //             );
    //         }
    //     }
    //     println!();
    // }
}
