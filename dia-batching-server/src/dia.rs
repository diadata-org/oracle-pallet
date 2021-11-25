use async_trait::async_trait;
use chrono::prelude::*;
use serde::Deserialize;
use std::error;
use rust_decimal::Decimal;

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
#[derive(Deserialize, Debug, Clone)]
pub struct Symbols {
	#[serde(rename(deserialize = "Symbols"))]
	pub symbols: Vec<String>,
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
#[derive(Deserialize, Debug, Clone)]
pub struct Quotation {
	#[serde(rename(deserialize = "Symbol"))]
	pub symbol: String,
	#[serde(rename(deserialize = "Name"))]
	pub name: String,
	#[serde(rename(deserialize = "Price"))]
	pub price: Decimal,
	#[serde(rename(deserialize = "PriceYesterday"))]
	pub price_yesterday: Decimal,
	#[serde(rename(deserialize = "VolumeYesterdayUSD"))]
	pub volume_yesterday: Decimal,
	#[serde(rename(deserialize = "Time"))]
	pub time: DateTime<Utc>,
}

impl Default for Quotation {
	fn default() -> Self {
		Self { time: Utc::now(), ..Default::default() }
	}
}

#[async_trait]
pub trait DiaApi {
	async fn get_symbols(&self) -> Result<Symbols, Box<dyn error::Error + Send + Sync>>;
	async fn get_quotation(
		&self,
		_: &str,
	) -> Result<Quotation, Box<dyn error::Error + Sync + Send>>;
}
pub struct Dia;

#[async_trait]
impl DiaApi for Dia {
	async fn get_quotation(
		&self,
		symbol: &str,
	) -> Result<Quotation, Box<dyn error::Error + Send + Sync>> {
		let r = reqwest::get(&format!("{}/{}", QUOTATION_ENDPOINT, symbol)).await?;
		let q: Quotation = r.json().await?;
		Ok(q)
	}

	async fn get_symbols(&self) -> Result<Symbols, Box<dyn error::Error + Sync + Send>> {
		let r = reqwest::get(SYMBOLS_ENDPOINT).await?;
		let s: Symbols = r.json().await?;
		Ok(s)
	}
}
