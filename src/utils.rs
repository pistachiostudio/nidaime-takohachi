use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    title: String,
    forecasts: Vec<Forecast>,
}

#[derive(Debug, Deserialize)]
struct Forecast {
    telop: String,
    detail: Detail,
}

#[derive(Debug, Deserialize)]
struct Detail {
    weather: String,
}

#[derive(Debug, Deserialize)]
struct StockQuote {
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: f64,
    #[serde(rename = "regularMarketChangePercent")]
    regular_market_change_percent: f64,
}

#[derive(Debug, Deserialize)]
struct YahooFinanceResponse {
    #[serde(rename = "quoteResponse")]
    quote_response: QuoteResponse,
}

#[derive(Debug, Deserialize)]
struct QuoteResponse {
    result: Vec<StockQuote>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Debug, Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Deserialize)]
struct Part {
    text: String,
}

pub fn get_what_today(month: u32, day: u32) -> String {
    let special_days: HashMap<(u32, u32), &str> = [
        ((1, 1), "元日 🎍"),
        ((2, 14), "バレンタインデー 💝"),
        ((3, 3), "ひな祭り 🎎"),
        ((4, 1), "エイプリルフール 🃏"),
        ((5, 5), "こどもの日 🎏"),
        ((7, 7), "七夕 🎋"),
        ((10, 31), "ハロウィン 🎃"),
        ((12, 24), "クリスマスイブ 🎄"),
        ((12, 25), "クリスマス 🎅"),
        ((12, 31), "大晦日 🎊"),
    ]
    .iter()
    .cloned()
    .collect();

    if let Some(event) = special_days.get(&(month, day)) {
        format!("今日は{}です！", event)
    } else {
        format!("{}月{}日です。", month, day)
    }
}

pub async fn get_weather(citycode: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let url = format!(
        "https://weather.tsukumijima.net/api/forecast/city/{}",
        citycode
    );

    let response = reqwest::get(&url).await?;
    let weather_data: WeatherResponse = response.json().await?;

    if let Some(today_forecast) = weather_data.forecasts.first() {
        let city_name = match citycode {
            "130010" => "東京",
            "060010" => "山形",
            _ => &weather_data.title,
        };

        Ok(format!(
            "**{}の天気**: {} - {}",
            city_name, today_forecast.telop, today_forecast.detail.weather
        ))
    } else {
        Ok("天気情報を取得できませんでした".to_string())
    }
}

pub async fn get_stock_price(ticker: &str) -> Result<(String, f64), Box<dyn Error + Send + Sync>> {
    let url = format!(
        "https://query1.finance.yahoo.com/v7/finance/quote?symbols={}",
        ticker
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await?;

    let finance_data: YahooFinanceResponse = response.json().await?;

    if let Some(quote) = finance_data.quote_response.result.first() {
        let change_percent = quote.regular_market_change_percent;
        let sign = if change_percent >= 0.0 { "+" } else { "" };
        let ratio_str = format!("{}{:.2}%", sign, change_percent);

        Ok((ratio_str, quote.regular_market_price))
    } else {
        Err("株価情報を取得できませんでした".into())
    }
}

pub async fn get_trivia(api_key: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-exp:generateContent?key={}",
        api_key
    );

    let prompt = "今日の面白い雑学を1つ、日本語で50文字以内で教えてください。";

    let request_body = json!({
        "contents": [{
            "parts": [{
                "text": prompt
            }]
        }]
    });

    let client = reqwest::Client::new();
    let response = client.post(&url).json(&request_body).send().await?;

    let gemini_response: GeminiResponse = response.json().await?;

    if let Some(candidate) = gemini_response.candidates.first()
        && let Some(part) = candidate.content.parts.first()
    {
        return Ok(part.text.clone());
    }

    Ok("今日の雑学: 知識は力なり！".to_string())
}
