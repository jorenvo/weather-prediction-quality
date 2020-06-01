#![warn(clippy::all)]

fn query_weather_com() -> reqwest::blocking::Response {
    println!("Querying...");
    let magic_cookie = "logatimLevel=INFO; speedpin=4G; akacd_NxtGen-DHLS=2177452799~rv=91~id=b5309a1374f3921da0b4e772431d157b; ci=TWC-Connection-Speed=4G&TWC-Locale-Group=US&TWC-Device-Class=desktop&X-Origin-Hint=Prod-IBM-LS&TWC-Network-Type=wifi&TWC-GeoIP-Country=US&TWC-GeoIP-Lat=37.7795&TWC-GeoIP-Long=-122.4195&Akamai-Connection-Speed=1000+&TWC-Privacy=usa-ccpa";
    let url = "https://weather.com/weather/tenday/l/San+Francisco+CA?canonicalCityId=dfdaba8cbe3a4d12a8796e1f7b1ccc7174b4b0a2d5ddb1c8566ae9f154fa638c";
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(url)
        .header("cookie", magic_cookie)
        .send()
        .unwrap();
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
    let selector = scraper::Selector::parse(".clickable.closed").unwrap();

    for day in fragment.select(&selector) {
        println!(
            "day: {}, high: {}, low: {}",
            select_first_inner(day, ".day-detail"),
            select_first_inner(day, ".temp span:nth-child(1)"),
            select_first_inner(day, ".temp span:nth-child(3)")
        );
    }
}

fn main() {
    let response = query_weather_com();
    scrape_info(&response.text().unwrap());
}
