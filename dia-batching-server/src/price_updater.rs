use crate::dia::{DiaApi, Quotation, Symbols};
use crate::storage::{CoinInfo, CoinInfoStorage};
use log::{error, info};
use std::{error::Error, sync::Arc};

pub async fn run_update_prices_loop<T>(
	storage: Arc<CoinInfoStorage>,
	rate: std::time::Duration,
	duration: std::time::Duration,
	api: T,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>>
where
	T: DiaApi + Send + Sync + 'static,
{
	let coins = Arc::clone(&storage);
	tokio::spawn(async move {
		loop {
			let time_elapsed = std::time::Instant::now();

			let coins = Arc::clone(&coins);

			update_prices(coins, &api, rate).await;

			tokio::time::delay_for(duration.saturating_sub(time_elapsed.elapsed())).await;
		}
	})
	.await?;
	Ok(())
}

async fn update_prices<T>(coins: Arc<CoinInfoStorage>, api: &T, rate: std::time::Duration)
where
	T: DiaApi + Send + Sync + 'static,
{
	if let Ok(Symbols { symbols }) = api.get_symbols().await {
		info!("No. of currencies to retrieve : {}", symbols.len());

		let mut currencies = vec![];

		for s in &symbols {
			if let Ok(Quotation { name, symbol, price, time, volume_yesterday, .. }) =
				api.get_quotation(s).await
			{
				let converted_price = convert_str_to_u128(&price.to_string());
				let converted_supply = convert_str_to_u128(&volume_yesterday.to_string());

				match (converted_price, converted_supply) {
					(Ok(x), Ok(y)) => {
						let coin_info = CoinInfo {
							name: name.into(),
							symbol: symbol.into(),
							price: x,
							last_update_timestamp: time.timestamp().unsigned_abs(),
							supply: y,
						};

						info!("Coin Price: {:#?}", price);
						info!("Coin Supply: {:#?}", volume_yesterday);
						info!("Coin Info : {:#?}", coin_info);

						currencies.push(coin_info);
					}
					_ => error!("Invalid input :{}", s),
				};
			} else {
				error!("Error while retrieving quotation for {}", s);
			}
			tokio::time::delay_for(rate).await;
		}
		coins.replace_currencies_by_symbols(currencies);
		info!("Currencies Updated");
	}
}
#[derive(Debug)]
pub enum ConvertingError {
	ParseIntError,
	InvalidInput,
}

fn convert_str_to_u128(input: &str) -> Result<u128, ConvertingError> {
	match input.split(".").collect::<Vec<_>>()[..] {
		[major] => Ok(major.parse::<u128>().map_err(|_| ConvertingError::ParseIntError)?
			* 10u128.pow(12 as u32)),

		[major, minor] => {
			let major_parsed_number =
				major.parse::<u128>().map_err(|_| ConvertingError::ParseIntError)?;

			let minor_parsed_number = precision_digits(minor).map_err(|e| e)?;

			Ok((major_parsed_number * 10u128.pow(12 as u32)).saturating_add(minor_parsed_number))
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

#[cfg(test)]
mod tests {
	use std::{collections::HashMap, error::Error, sync::Arc};

	use async_trait::async_trait;
	use chrono::Utc;
	use rust_decimal_macros::dec;

	use super::*;

	struct MockDia<'a> {
		quotation: HashMap<&'a str, Quotation>,
	}

	impl<'a> MockDia<'a> {
		pub fn new() -> Self {
			let mut quotation = HashMap::new();
			quotation.insert(
				"BTC",
				Quotation {
					name: "BTC".into(),
					price: dec!(1.000000000000),
					price_yesterday: dec!(1.000000000000),
					symbol: "BTC".into(),
					time: Utc::now(),
					volume_yesterday: dec!(1.000000000000),
				},
			);
			quotation.insert(
				"ETH",
				Quotation {
					name: "ETH".into(),
					price: dec!(1.000000000000),
					price_yesterday: dec!(1.000000000000),
					symbol: "ETH".into(),
					time: Utc::now(),
					volume_yesterday: dec!(1.000000000000),
				},
			);
			quotation.insert(
				"ADA",
				Quotation {
					name: "ADA".into(),
					price: dec!(0),
					price_yesterday: dec!(1.000000000000),
					symbol: "ADA".into(),
					time: Utc::now(),
					volume_yesterday: dec!(0.123456789012345),
				},
			);
			quotation.insert(
				"XRP",
				Quotation {
					name: "XRP".into(),
					price: dec!(123456789.123456789012345),
					price_yesterday: dec!(1.000000000000),
					symbol: "XRP".into(),
					time: Utc::now(),
					volume_yesterday: dec!(298134760),
				},
			);
			quotation.insert(
				"DOGE",
				Quotation {
					name: "DOGE".into(),
					price: dec!(1.000000000001),
					price_yesterday: dec!(1.000000000000),
					symbol: "DOGE".into(),
					time: Utc::now(),
					volume_yesterday: dec!(0.000000000001),
				},
			);
			Self { quotation }
		}
	}

	#[async_trait]
	impl<'a> DiaApi for MockDia<'a> {
		async fn get_quotation(
			&self,
			symbol: &str,
		) -> Result<Quotation, Box<dyn Error + Send + Sync>> {
			Ok(self.quotation.get(symbol).ok_or("Error Finding Quotation".to_string())?.clone())
		}

		async fn get_symbols(&self) -> Result<Symbols, Box<dyn Error + Send + Sync>> {
			Ok(Symbols {
				symbols: vec![
					"BTC".into(),
					"ETH".into(),
					"ADA".into(),
					"XRP".into(),
					"DOGE".into(),
				],
			})
		}
	}
	#[tokio::test]
	async fn test_update_prices() {
		let mock_api = MockDia::new();
		let storage = Arc::new(CoinInfoStorage::default());
		let coins = Arc::clone(&storage);
		update_prices(coins, &mock_api, std::time::Duration::from_secs(1)).await;

		let c = storage.get_currencies_by_symbols(&["BTC", "ETH", "ADA", "XRP"]);

		assert_eq!(4, c.len());

		assert_eq!(c[1].price, 1000000000000);

		assert_eq!(c[1].name, "ETH");
	}

	#[tokio::test]
	async fn test_update_prices_non_existent() {
		let mock_api = MockDia::new();
		let storage = Arc::new(CoinInfoStorage::default());
		let coins = Arc::clone(&storage);
		update_prices(coins, &mock_api, std::time::Duration::from_secs(1)).await;

		let c = storage.get_currencies_by_symbols(&["BTCCash", "ETHCase"]);

		assert_eq!(0, c.len());
	}

	#[tokio::test]
	async fn test_update_prices_one_available() {
		let mock_api = MockDia::new();
		let storage = Arc::new(CoinInfoStorage::default());
		let coins = Arc::clone(&storage);
		update_prices(coins, &mock_api, std::time::Duration::from_secs(1)).await;

		let c = storage.get_currencies_by_symbols(&["BTC", "ETHCase"]);

		assert_eq!(1, c.len());

		assert_eq!(c[0].price, 1000000000000);

		assert_eq!(c[0].name, "BTC");
	}

	#[tokio::test]
	async fn test_update_prices_get_nothing() {
		let mock_api = MockDia::new();
		let storage = Arc::new(CoinInfoStorage::default());
		let coins = Arc::clone(&storage);
		update_prices(coins, &mock_api, std::time::Duration::from_secs(1)).await;

		let c = storage.get_currencies_by_symbols::<&str>(&[]);

		assert_eq!(0, c.len());
	}

	#[tokio::test]
	async fn test_update_prices_get_integers() {
		let mock_api = MockDia::new();
		let storage = Arc::new(CoinInfoStorage::default());
		let coins = Arc::clone(&storage);

		update_prices(coins, &mock_api, std::time::Duration::from_secs(1)).await;

		let c = storage.get_currencies_by_symbols(&["123"]);

		assert_eq!(0, c.len());
	}

	#[tokio::test]
	async fn test_convert_result() {
		let mock_api = MockDia::new();
		let storage = Arc::new(CoinInfoStorage::default());
		let coins = Arc::clone(&storage);

		update_prices(coins, &mock_api, std::time::Duration::from_secs(1)).await;

		let c = storage.get_currencies_by_symbols(&["ADA", "XRP", "DOGE"]);

		assert_eq!(c[0].price, 0);
		assert_eq!(c[0].supply, 123456789012);

		assert_eq!(c[1].price, 123456789123456789012);
		assert_eq!(c[1].supply, 298134760000000000000);

		assert_eq!(c[2].price, 1000000000001);
		assert_eq!(c[2].supply, 1);

		assert_eq!(c[0].name, "ADA");
		assert_eq!(c[1].name, "XRP");
		assert_eq!(c[2].name, "DOGE");
	}
}
