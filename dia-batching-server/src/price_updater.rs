use crate::storage::{CoinInfo, CoinInfoStorage};

use chrono::prelude::*;
use log::{error, info};
use serde::Deserialize;

use std::sync::Arc;

const SYMBOLS_ENDPOINT: &str = "https://api.diadata.org/v1/symbols";
/// ### Symbols
///
/// `GET : https://api.diadata.org/v1/symbols`
///
/// Get most recent information on the currency corresponding to symbol.
///
/// Example:
/// https://api.diadata.org/v1/symbols
///
/// Response:
/// ```ignore
/// {
/// 	"Symbols":["BTC",...]
/// }
/// ```
#[derive(Deserialize, Debug)]
struct Symbols {
	#[serde(rename(deserialize = "Symbols"))]
	symbols: Vec<String>,
}

const QUOTATION_ENDPOINT: &str = "https://api.diadata.org/v1/quotation";
/// ### Quotation
///
/// `GET : https://api.diadata.org/v1/quotation/:symbol`
///
/// Get most recent information on the currency corresponding to symbol.
///
/// Example:
/// https://api.diadata.org/v1/quotation/BTC
///
/// Response:
/// ```ignore
/// {
///		"Symbol":"BTC",
///		"Name":"Bitcoin",
///		"Price":9777.19339776667,
///		"PriceYesterday":9574.416265039981,
///		"VolumeYesterdayUSD":298134760.8811487,
///		"Source":"diadata.org",
///		"Time":"2020-05-19T08:41:12.499645584Z",
///		"ITIN":"DXVPYDQC3"
/// }
/// ```
#[derive(Deserialize, Debug)]
struct Quotation {
	#[serde(rename(deserialize = "Symbol"))]
	symbol: String,
	#[serde(rename(deserialize = "Name"))]
	name: String,
	#[serde(rename(deserialize = "Price"))]
	price: f64,
	#[serde(rename(deserialize = "PriceYesterday"))]
	price_yesterday: f64,
	#[serde(rename(deserialize = "VolumeYesterdayUSD"))]
	volume_yesterday: f64,
	#[serde(rename(deserialize = "Time"))]
	time: DateTime<Utc>,
}

pub async fn run_update_prices_loop(
	storage: Arc<CoinInfoStorage>,
	rate: std::time::Duration,
	duration: std::time::Duration,
) {
	let coins = Arc::clone(&storage);

	tokio::spawn(async move {
		loop {
			let time_elapsed = std::time::Instant::now();
			if let Ok(r) = reqwest::get(SYMBOLS_ENDPOINT).await {
				if let Ok(Symbols { symbols }) = r.json().await {
					info!("No. of currencies to retrieve : {}", symbols.len());

					let mut currencies = vec![];

					for s in &symbols {
						if let Ok(quote) =
							reqwest::get(&format!("{}/{}", QUOTATION_ENDPOINT, s)).await
						{
							if let Ok(Quotation {
								name,
								symbol,
								price,
								time,
								volume_yesterday,
								..
							}) = quote.json().await
							{
								let coin_info = CoinInfo {
									name: name.into(),
									symbol: symbol.into(),
									price: convert_str_to_u64(&price.to_string()), // Converting f64 to u64
									last_update_timestamp: time.timestamp().unsigned_abs(),
									supply: convert_str_to_u64(&volume_yesterday.to_string()), // Converting f64 to u64
								};

								info!("Coin Price: {:#?}", price);
								info!("Coin Supply: {:#?}", volume_yesterday);
								info!("Coin Info : {:#?}", coin_info);

								currencies.push(coin_info);
							} else {
								error!("Error while retrieving quotation for {}", s);
							}
						}
						tokio::time::delay_for(rate).await;
					}

					coins.replace_currencies_by_symbols(currencies);
					info!("Currencies Updated");
				}
			}
			tokio::time::delay_for(duration.saturating_sub(time_elapsed.elapsed())).await;
		}
	})
	.await
	.unwrap();
}

// TODO : Converting their floating pricing into u64
fn convert_str_to_u64(input: &str) -> u64 {
	match input.split(".").collect::<Vec<_>>()[..] {
		[major] => major.parse::<u64>().unwrap(),
		[major, minor] => (major.parse::<u128>().unwrap() * 10u128.pow(minor.len() as u32))
			.saturating_add(minor.parse::<u128>().unwrap()) as u64,
		_ => 0,
	}
}
