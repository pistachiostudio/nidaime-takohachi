use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    title: String,
    forecasts: Vec<Forecast>,
    location: Location,
}

#[derive(Debug, Deserialize)]
struct Forecast {
    telop: String,
    detail: Detail,
    temperature: TemperatureAll,
    #[serde(rename = "chanceOfRain")]
    chance_of_rain: ChanceOfRain,
}

#[derive(Debug, Deserialize)]
struct Detail {
    weather: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TemperatureAll {
    // min は null であることがあり、初代タコ八でも採用していないので外している
    max: Option<Temperature>,
}

#[derive(Debug, Deserialize)]
struct Temperature {
    celsius: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChanceOfRain {
    #[serde(rename = "T06_12")]
    morning: Option<String>,
    #[serde(rename = "T12_18")]
    afternoon: Option<String>,
    #[serde(rename = "T18_24")]
    night: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Location {
    city: String,
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

    let response = match reqwest::get(&url).await {
        Ok(resp) => resp,
        Err(e) => {
            println!(
                "Weather API HTTP request failed - URL: {}, Error: {}",
                url, e
            );
            return Err(Box::new(e));
        }
    };

    let status = response.status();
    if !status.is_success() {
        println!(
            "Weather API returned non-success status: {} for URL: {}",
            status, url
        );
    }

    let response_text = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            println!("Failed to get response text - Error: {}", e);
            return Err(Box::new(e));
        }
    };

    let weather_data: WeatherResponse = match serde_json::from_str(&response_text) {
        Ok(data) => data,
        Err(e) => {
            println!("Failed to parse weather API JSON response - Error: {}", e);
            println!("Response text: {}", response_text);
            return Err(Box::new(e));
        }
    };

    if let Some(today_forecast) = weather_data.forecasts.first() {
        let city_name = &weather_data.location.city;

        // 天気情報から全角スペースを削除
        let weather = today_forecast
            .detail
            .weather
            .as_ref()
            .map(|w| w.replace("　", ""))
            .unwrap_or_else(|| today_forecast.telop.clone());

        // 最高気温を取得
        let max_temp = today_forecast
            .temperature
            .max
            .as_ref()
            .and_then(|t| t.celsius.as_ref())
            .map(|c| c.as_str())
            .unwrap_or("--");

        // 降水確率を取得
        let morning_rain = today_forecast
            .chance_of_rain
            .morning
            .as_ref()
            .map(|r| r.as_str())
            .unwrap_or("--%");
        let afternoon_rain = today_forecast
            .chance_of_rain
            .afternoon
            .as_ref()
            .map(|r| r.as_str())
            .unwrap_or("--%");
        let night_rain = today_forecast
            .chance_of_rain
            .night
            .as_ref()
            .map(|r| r.as_str())
            .unwrap_or("--%");

        Ok(format!(
            "- {}: {}\n  - 🌡️ 最高気温: {} ℃\n  - ☔ 朝: {} | 昼: {} | 晩: {}",
            city_name, weather, max_temp, morning_rain, afternoon_rain, night_rain
        ))
    } else {
        println!(
            "Warning: Weather API returned empty forecast data for citycode: {}",
            citycode
        );
        Ok("天気情報を取得できませんでした".to_string())
    }
}

pub async fn get_stock_price(ticker: &str) -> Result<(String, f64), Box<dyn Error + Send + Sync>> {
    use yahoo_finance_api as yahoo;

    let provider = match yahoo::YahooConnector::new() {
        Ok(provider) => provider,
        Err(e) => {
            println!("Failed to create Yahoo Finance connector - Error: {}", e);
            return Err(Box::new(e));
        }
    };

    let response = match provider.get_latest_quotes(ticker, "1d").await {
        Ok(response) => response,
        Err(e) => {
            println!(
                "Failed to get stock quotes for ticker: {} - Error: {}",
                ticker, e
            );
            return Err(Box::new(e));
        }
    };

    let quotes = response.quotes()?;

    if let Some(quote) = quotes.first() {
        let current_price = quote.close;
        let previous_close = quote.open;
        let change_percent = if previous_close != 0.0 {
            ((current_price - previous_close) / previous_close) * 100.0
        } else {
            0.0
        };

        let sign = if change_percent >= 0.0 { "+" } else { "" };
        let ratio_str = format!("{}{:.2}%", sign, change_percent);

        Ok((ratio_str, current_price))
    } else {
        println!("Warning: No stock data found for ticker: {}", ticker);
        Err("株価情報を取得できませんでした".into())
    }
}

pub async fn get_trivia(api_key: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-exp:generateContent?key={}",
        api_key
    );

    let request_body = json!({
        "contents": [{
            "parts": [{
                "text": "あなたはあらゆる分野からランダムに興味深い雑学を紹介するエキスパートです。 \
                        以下の分野から毎回ランダムに異なるテーマを選び、約400文字の日本語で雑学を1つ紹介してください。 \
                        対象分野：動植物、生物学、宇宙、地理、歴史、哲学、科学、物理学、化学、数学、言語、文学、芸術、音楽、 \
                        映画、カルチャー、食文化、スポーツ、テクノロジー、心理学、社会学、経済学、建築、医学、人体、民俗学、都市伝説など \
                        紹介する雑学は毎回前回と異なる分野から選んでください。雑学の内容はマニアックであっても構いません。 \
                        冒頭に挨拶や前置きは一切不要です。冒頭に分野を記載することも不要で、本文のみ記載してください。"
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
