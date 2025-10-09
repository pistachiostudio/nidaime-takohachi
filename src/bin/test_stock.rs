use chrono::{TimeZone, Utc};
use chrono_tz::Asia::Tokyo;
use std::env;
use std::error::Error;
use yahoo_finance_api as yahoo;

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

async fn get_stock_price(ticker: &str) -> Result<(String, String), Box<dyn Error + Send + Sync>> {
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

    // 取得したデータをループで表示
    println!("\n取得したデータ (最大5日分):");
    for (i, quote) in quotes.iter().enumerate() {
        let date = Utc.timestamp_opt(quote.timestamp, 0).unwrap();
        let date_tokyo = date.with_timezone(&Tokyo);
        println!(
            "[Day {}] Date: {}, close: {:.2}, low: {:.2}, high: {:.2}, open: {:.2}, volume: {}, adjclose: {:.2}",
            i + 1,
            date_tokyo.format("%Y-%m-%d %H:%M:%S JST"),
            quote.close,
            quote.low,
            quote.high,
            quote.open,
            quote.volume,
            quote.adjclose
        );
    }
    println!();
    let stock_yesterday = quotes[0].close;
    let stock_today = quotes[1].close;

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

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // デフォルトは日経平均
    let ticker = if args.len() > 1 { &args[1] } else { "^N225" };

    let ticker_name = match ticker {
        "^N225" => "日経平均株価",
        "^DJI" => "ダウ平均株価",
        "^GSPC" => "S&P 500",
        "^IXIC" => "NASDAQ総合指数",
        _ => ticker,
    };

    println!("株価情報を取得中: {} ({})", ticker_name, ticker);
    println!("----------------------------------------");

    match get_stock_price(ticker).await {
        Ok((ratio_str, price)) => {
            println!("✅ 取得成功!");
            println!("銘柄: {}", ticker_name);
            println!("現在値: ${}", price);
            println!("前日比: {}", ratio_str);
        }
        Err(e) => {
            println!("❌ エラーが発生しました: {}", e);
        }
    }
}
