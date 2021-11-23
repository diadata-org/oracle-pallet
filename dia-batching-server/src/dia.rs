use async_trait::async_trait;
use chrono::prelude::*;
use serde::{de::Error, Deserialize, Deserializer};
use serde_json::Number;
use std::{error, fmt};

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
	#[serde(rename(deserialize = "Price"), deserialize_with = "convert")]
	pub price: u128,
	#[serde(rename(deserialize = "PriceYesterday"), deserialize_with = "convert")]
	pub price_yesterday: u128,
	#[serde(rename(deserialize = "VolumeYesterdayUSD"), deserialize_with = "convert")]
	pub volume_yesterday: u128,
	#[serde(rename(deserialize = "Time"))]
	pub time: DateTime<Utc>,
}

#[derive(Debug)]
pub enum ConvertingError {
	ParseIntError,
	InvalidInput,
}

impl error::Error for ConvertingError {}
impl fmt::Display for ConvertingError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use ConvertingError::*;
		match self {
			ParseIntError => write!(f, "ParseIntError"),
			InvalidInput => write!(f, "InvalidInput"),
		}
	}
}

fn convert<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
	D: Deserializer<'de>,
{
	fn convert_str_to_u128(input: &str) -> Result<u128, ConvertingError> {
		match input.split(".").collect::<Vec<_>>()[..] {
			[major] => Ok(major.parse::<u128>().map_err(|_| ConvertingError::ParseIntError)?
				* 10u128.pow(12 as u32)),

			[major, minor] => {
				let major_parsed_number =
					major.parse::<u128>().map_err(|_| ConvertingError::ParseIntError)?;

				let minor_parsed_number = precision_digits(minor).map_err(|e| e)?;

				Ok((major_parsed_number * 10u128.pow(12 as u32))
					.saturating_add(minor_parsed_number))
			}
			// ultimately it won't run to this option
			_ => Err(ConvertingError::InvalidInput),
		}
	}

	fn precision_digits(minor: &str) -> Result<u128, ConvertingError> {
		const PRECISION_IN_DIGITS: usize = 12;

		let range =
			if minor.len() < PRECISION_IN_DIGITS { ..minor.len() } else { ..PRECISION_IN_DIGITS };

		let twelve_digits = minor.get(range).ok_or(ConvertingError::ParseIntError)?;

		let parsed_number =
			twelve_digits.parse::<u128>().map_err(|_| ConvertingError::ParseIntError)?;

		if minor.len() < PRECISION_IN_DIGITS {
			return Ok(parsed_number * 10u128.pow((PRECISION_IN_DIGITS - minor.len()) as u32));
		}

		Ok(parsed_number) 
	}

	let result = convert_str_to_u128(&Number::deserialize(deserializer)?.to_string())
		.map_err(|_| ConvertingError::InvalidInput)
		.map_err(D::Error::custom)?;

	Ok(result)
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

#[test]
fn quotation_data_price() {
	let quotation_result = serde_json::from_str::<Quotation>(
		r#"
	 {
		"Symbol":"BTC",
		"Name":"Bitcoin",
		"Price":98765.123456789012345,
		"PriceYesterday":0.123456789012345,
		"VolumeYesterdayUSD":298134760,
		"Source":"diadata.org",
		"Time":"2020-05-19T08:41:12.499645584Z",
		"ITIN":"DXVPYDQC3"
	 }
	"#,
	)
	.unwrap();

	let quotation_data = Quotation {
		symbol: "BTC".into(),
		name: "BTC".into(),
		price: 98765123456789012,      //without 345
		price_yesterday: 123456789012, // only decimal is stored, if numbers, zero couldn't stored at most left
		time: Utc::now(),
		volume_yesterday: 298134760000000000000, // twelve 0s
	};

	assert_eq!(quotation_result.price, quotation_data.price);
	assert_eq!(quotation_result.price_yesterday, quotation_data.price_yesterday);
	assert_eq!(quotation_result.volume_yesterday, quotation_data.volume_yesterday);
}
#[test]
fn quotation_data_price_with_zeros_at_front() {
	let quotation_result = serde_json::from_str::<Quotation>(
		r#"
	 {
		"Symbol":"BTC",
		"Name":"Bitcoin",
		"Price":0,
		"PriceYesterday":1.000000000001,
		"VolumeYesterdayUSD":0.000000000001,
		"Source":"diadata.org",
		"Time":"2020-05-19T08:41:12.499645584Z",
		"ITIN":"DXVPYDQC3"
	 }
	"#,
	).unwrap();

	let quotation_data = Quotation {
		symbol: "BTC".into(),
		name: "BTC".into(),
		price: 0, //price = 0
		price_yesterday: 1000000000001,
		time: Utc::now(),
		volume_yesterday: 1, // with 0s, and only value at most right
	};

	assert_eq!(quotation_result.price, quotation_data.price);
	assert_eq!(quotation_result.price_yesterday, quotation_data.price_yesterday);
	assert_eq!(quotation_result.volume_yesterday, quotation_data.volume_yesterday);
}

#[tokio::test]
async fn test_quotation() {
	let symbol = "BTC";
	let r = reqwest::get(&format!("{}/{}", QUOTATION_ENDPOINT, symbol)).await.unwrap();
	let q: Quotation = r.json().await.unwrap();
	println!("RESPOND: {:#?}", q);
	// Example:
	// https://api.diadata.org/v1/quotation/BTC
}
