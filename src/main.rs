#![warn(clippy::all)]
use chrono::{Datelike, Local, NaiveDate};
use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::{Read, Write};

const DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Serialize, Deserialize, Debug)]
struct TempHiLo {
    low: String,
    high: String,
}

struct Prediction {
    date: String,
    temp: TempHiLo,
}

#[derive(Serialize, Deserialize, Debug)]
struct SavedPredictions {
    predictions: BTreeMap<String, Vec<TempHiLo>>,
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
        );
        let high = select_first_inner(prediction, "[data-testid=\"TemperatureValue\"]");
        let temp = TempHiLo { low, high };

        predictions.push(Prediction { date, temp });
    }

    predictions
}

/// Returns the first positional argument sent to this process. If there are no
/// positional arguments, then this returns an error.
fn get_first_arg() -> OsString {
    env::args_os()
        .nth(1)
        .expect("expected data store path as first argument")
}

fn read_data_store() -> String {
    let mut data_store_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(get_first_arg())
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

fn write_data_store(serialized: String) {
    let mut data_store_file = OpenOptions::new()
        .write(true)
        .open(get_first_arg())
        .unwrap();
    data_store_file.write_all(serialized.as_bytes()).unwrap();
}

fn main() {
    let data_store = read_data_store();

    let response = query_weather_com();
    let predictions = scrape_info(&response.text().unwrap());

    println!("Disk data store: {}", data_store);
    let mut saved_predictions: SavedPredictions = serde_json::from_str(&data_store).unwrap();
    for prediction in predictions {
        let prev_values = saved_predictions
            .predictions
            .entry(prediction.date)
            .or_insert(vec![]);
        prev_values.push(prediction.temp);
    }

    for (date, temperatures) in &saved_predictions.predictions {
        println!("{}: {:?}", date, temperatures);
    }

    println!(
        "serialized: {}",
        serde_json::to_string(&saved_predictions).unwrap()
    );

    write_data_store(serde_json::to_string(&saved_predictions).unwrap());

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
