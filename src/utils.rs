use rand::seq::SliceRandom;
use regex::Regex;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::json;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct WeatherResponse {
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

pub async fn get_what_today(month: u32, day: u32) -> String {
    // Wikimedia Foundation User-Agent Policy 準拠の User-Agent
    // 参考: https://foundation.wikimedia.org/wiki/Policy:Wikimedia_Foundation_User-Agent_Policy
    let app_name = "nidaime-takohachi";
    let app_version = "0.1.0";
    let app_url = "https://github.com/pistachiostudio/nidaime-takohachi";
    let app_contact = "info@pistachiostudio.net";
    let user_agent = format!(
        "{}/{} (+{}; {})",
        app_name, app_version, app_url, app_contact
    );

    let base_url = "https://ja.wikipedia.org/wiki/Wikipedia:";
    let uri = format!("今日は何の日_{}月", month);
    let url = format!("{}{}", base_url, urlencoding::encode(&uri));

    // HTTP リクエストを送信
    let client = reqwest::Client::new();
    let response = match client
        .get(&url)
        .header("User-Agent", user_agent)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            println!(
                "Wikipedia への HTTP リクエストに失敗しました - URL: {}, エラー: {}",
                url, e
            );
            return format!("{}月{}日です。", month, day);
        }
    };

    let html = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            println!("レスポンステキストの取得に失敗しました - エラー: {}", e);
            return format!("{}月{}日です。", month, day);
        }
    };

    // HTML をパース
    let document = Html::parse_document(&html);

    // 指定された日付の見出しを探す（h2 要素）
    let target_text = format!("{}月{}日", month, day);
    let h2_selector = match Selector::parse("h2") {
        Ok(sel) => sel,
        Err(_) => return format!("{}月{}日です。", month, day),
    };

    let mut h2_element = None;
    for h2 in document.select(&h2_selector) {
        let text = h2.text().collect::<String>();
        if text == target_text {
            h2_element = Some(h2);
            break;
        }
    }

    let h2_element = match h2_element {
        Some(h) => h,
        None => {
            return format!("{}月{}日です。", month, day);
        }
    };

    // h2 の次の要素を探して ul を見つける
    let mut current = match h2_element.parent() {
        Some(parent) => parent,
        None => return format!("{}月{}日です。", month, day),
    };

    let ul_element = loop {
        current = match current.next_sibling() {
            Some(sibling) => sibling,
            None => {
                return format!("{}月{}日です。", month, day);
            }
        };

        if current.value().is_element() && current.value().as_element().unwrap().name() == "ul" {
            break current;
        }
    };

    // ul 内のすべての li 要素を取得
    let li_selector = match Selector::parse("li") {
        Ok(sel) => sel,
        Err(_) => return format!("{}月{}日です。", month, day),
    };

    // ul_element から直接 li 要素を検索
    let items: Vec<String> = document
        .select(&li_selector)
        .filter_map(|li| {
            // 親要素を辿って ul_element の子孫かチェック
            let mut ancestor = li.parent();
            let mut is_descendant = false;
            while let Some(node) = ancestor {
                if std::ptr::eq(node.value(), ul_element.value()) {
                    is_descendant = true;
                    break;
                }
                ancestor = node.parent();
            }

            if !is_descendant {
                return None;
            }

            // HTML タグを除去してテキストを取得
            let re = Regex::new(r"<[^>]+>").unwrap();
            let html = li.html();
            let text = re.replace_all(&html, "").trim().to_string();

            if text.is_empty() { None } else { Some(text) }
        })
        .collect();

    if items.is_empty() {
        println!("項目が見つかりませんでした");
        return format!("{}月{}日です。", month, day);
    }

    // ランダムに1つ選択
    let mut rng = rand::thread_rng();
    let selected = items.choose(&mut rng).unwrap();

    selected.to_string()
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
            .as_deref()
            .unwrap_or("--%");
        let afternoon_rain = today_forecast
            .chance_of_rain
            .afternoon
            .as_deref()
            .unwrap_or("--%");
        let night_rain = today_forecast
            .chance_of_rain
            .night
            .as_deref()
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

fn format_price_with_comma(price: f64) -> String {
    let price_str = format!("{:.1}", price);
    let parts: Vec<&str> = price_str.split('.').collect();
    let integer_part = parts[0];
    let decimal_part = parts.get(1).unwrap_or(&"0");

    let mut result = String::new();
    let chars: Vec<char> = integer_part.chars().collect();
    let len = chars.len();

    for (i, c) in chars.iter().enumerate() {
        result.push(*c);
        let remaining = len - i - 1;
        if remaining > 0 && remaining.is_multiple_of(3) {
            result.push(',');
        }
    }

    format!("{}.{}", result, decimal_part)
}

pub async fn get_stock_price(
    ticker: &str,
) -> Result<(String, String), Box<dyn Error + Send + Sync>> {
    use yahoo_finance_api as yahoo;

    let provider = match yahoo::YahooConnector::new() {
        Ok(provider) => provider,
        Err(e) => {
            println!("Failed to create Yahoo Finance connector - Error: {}", e);
            return Err(Box::new(e));
        }
    };

    let response = match provider.get_latest_quotes(ticker, "5d").await {
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

    // 最低2日分のデータが必要
    if quotes.len() < 2 {
        println!("Warning: Not enough stock data for ticker: {}", ticker);
        return Err("株価情報を取得できませんでした".into());
    }

    // 配列の最後から2番目（昨日）と最後（今日）の終値を取得
    let len = quotes.len();
    let stock_yesterday = quotes[len - 2].close;
    let stock_today = quotes[len - 1].close;

    // 差分を計算し、小数点第1位で四捨五入
    let day_before_ratio = ((stock_today - stock_yesterday) * 10.0).round() / 10.0;

    // 符号付きでカンマ区切りの価格差分をフォーマット
    let price_change_str = if day_before_ratio >= 0.0 {
        format!("+{}円", format_price_with_comma(day_before_ratio))
    } else {
        format!("{}円", format_price_with_comma(day_before_ratio))
    };

    // 価格変動を結合
    let ratio_str = format!("({})", price_change_str);

    // 現在価格をカンマ区切りでフォーマット
    let formatted_price = format_price_with_comma(stock_today);

    Ok((ratio_str, formatted_price))
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
