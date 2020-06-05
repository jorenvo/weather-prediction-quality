#![warn(clippy::all)]

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

fn scrape_info(html: &str) {
    let fragment = scraper::Html::parse_fragment(html);
    let selector = scraper::Selector::parse("[data-testid=\"DetailsSummary\"]").unwrap();

    for day in fragment.select(&selector) {
        println!(
            "day: {}, high: {}, low: {}",
            select_first_inner(day, "[data-testid=\"daypartName\""),
            select_first_inner(day, "[data-testid=\"TemperatureValue\"]"),
            select_first_inner(
                day,
                "[data-testid=\"lowTempValue\"] > [data-testid=\"TemperatureValue\"]"
            ),
        );
    }
}

fn main() {
    let response = query_weather_com();
    scrape_info(&response.text().unwrap());
}
