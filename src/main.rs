#![warn(clippy::all)]
use chrono::{Datelike, Duration, Local, NaiveDate};
use csv::{ReaderBuilder, WriterBuilder};
use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fmt::Debug;
use std::fs::File;

#[derive(Debug)]
struct TempHiLo {
    low: String,
    high: String,
}

struct Prediction {
    date: NaiveDate,
    temp: TempHiLo,
}

fn query_weather_com() -> reqwest::blocking::Response {
    println!("Querying...");
    let url = "https://weather.com/weather/tenday/l/San+Francisco+CA?canonicalCityId=dfdaba8cbe3a4d12a8796e1f7b1ccc7174b4b0a2d5ddb1c8566ae9f154fa638c";
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
        .expect("expected csv path as first argument")
}

fn main() {
    const DATE_FORMAT: &str = "%Y-%m-%d";
    let response = query_weather_com();
    let predictions = scrape_info(&response.text().unwrap());

    let mut predictions_map: BTreeMap<NaiveDate, Vec<TempHiLo>> = BTreeMap::new(); // TODO read this from file

    for prediction in predictions {
        let mut prev_values = predictions_map.entry(prediction.date).or_insert(vec![]);
        prev_values.push(prediction.temp);
    }

    for (date, temperatures) in predictions_map {
        println!("{}: {:?}", date, temperatures);
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
