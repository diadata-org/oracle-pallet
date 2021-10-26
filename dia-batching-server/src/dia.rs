use std::error::Error;

use async_trait::async_trait;
use chrono::prelude::*;
use serde::Deserialize;

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
	pub price: f64,
	#[serde(rename(deserialize = "PriceYesterday"))]
	pub price_yesterday: f64,
	#[serde(rename(deserialize = "VolumeYesterdayUSD"))]
	pub volume_yesterday: f64,
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
	type Symbols;
	type Quotation;

	async fn get_symbols(&self) -> Result<Self::Symbols, Box<dyn Error + Sync + Send>>;
	async fn get_quotation(&self, _: &str)
		-> Result<Self::Quotation, Box<dyn Error + Sync + Send>>;
}
pub struct Dia;
unsafe impl Send for Dia {}
unsafe impl Sync for Dia {}
pub enum DiaError {
	JsonParse,
	RequestFailed,
}

#[async_trait]
impl DiaApi for Dia {
	type Symbols = Symbols;
	type Quotation = Quotation;

	async fn get_quotation(
		&self,
		symbol: &str,
	) -> Result<Self::Quotation, Box<dyn Error + Sync + Send>> {
		match reqwest::get(&format!("{}/{}", QUOTATION_ENDPOINT, symbol)).await {
			Ok(r) => match r.json().await {
				Ok(q) => Ok(q),

				Err(e) => Err(Box::new(e)),
			},

			Err(e) => Err(Box::new(e)),
		}
	}

	async fn get_symbols(&self) -> Result<Self::Symbols, Box<dyn Error + Sync + Send>> {
		match reqwest::get(SYMBOLS_ENDPOINT).await {
			Ok(r) => match r.json().await {
				Ok(q) => Ok(q),

				Err(e) => Err(Box::new(e)),
			},
			Err(e) => Err(Box::new(e)),
		}
	}
}
