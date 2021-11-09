use crate::dia::{DiaApi, Quotation, Symbols};
use crate::storage::{CoinInfo, CoinInfoStorage};
use log::{error, info};
use std::io::Error;
use std::sync::Arc;
use tokio::task::JoinError;

pub async fn run_update_prices_loop<T>(
	storage: Arc<CoinInfoStorage>,
	rate: std::time::Duration,
	duration: std::time::Duration,
	api: T,
) -> Result<(), JoinError>
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
				let coin_info = CoinInfo {
					name: name.into(),
					symbol: symbol.into(),
					price: convert_str_to_u64(&price.to_string()).unwrap(), // Converting f64 to u64
					last_update_timestamp: time.timestamp().unsigned_abs(),
					supply: convert_str_to_u64(&volume_yesterday.to_string()).unwrap(), // Converting f64 to u64
				};

				info!("Coin Price: {:#?}", price);
				info!("Coin Supply: {:#?}", volume_yesterday);
				info!("Coin Info : {:#?}", coin_info);

				currencies.push(coin_info);
			} else {
				error!("Error while retrieving quotation for {}", s);
			}
			tokio::time::delay_for(rate).await;
		}
		coins.replace_currencies_by_symbols(currencies);
		info!("Currencies Updated");
	}
}

fn convert_str_to_u64(input: &str) -> Result<u64, Error> {
	match input.split(".").collect::<Vec<_>>()[..] {
		[major] => Ok(major.parse::<u64>().unwrap() * 10u64.pow(6 as u32)),
		[major, minor] => {
			let c = (major.parse::<u64>().unwrap() * 10u64.pow(6 as u32))
				.saturating_add(precision_digits(minor).unwrap());
			Ok(c)
		}
		// ultimately it won't run to this option
		_ => Ok(0),
	}
}

fn precision_digits(minor: &str) -> Result<u64, Error> {
	let minor: Vec<_> = minor.split("").filter(|minor| !minor.is_empty()).collect();
	let mut six_digit = Vec::new();
	match minor.len() {
		0..=5 => {
			let remaining_empty = 6 - minor.len();
			for i in 0..minor.len() {
				six_digit.push(minor[i])
			}

			let p = six_digit.join("").parse::<u64>().unwrap() * 10u64.pow(remaining_empty as u32);
			Ok(p)
		}
		_ => {
			for i in 0..6 {
				six_digit.push(minor[i])
			}

			let p = six_digit.join("").parse::<u64>().unwrap();
			Ok(p)
		}
	}
}

#[cfg(test)]
mod tests {
	use std::{collections::HashMap, error::Error, sync::Arc};

	use async_trait::async_trait;
	use chrono::Utc;

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
					price: 1.0,
					price_yesterday: 1.0,
					symbol: "BTC".into(),
					time: Utc::now(),
					volume_yesterday: 1.0,
				},
			);
			quotation.insert(
				"ETH",
				Quotation {
					name: "ETH".into(),
					price: 1.0,
					price_yesterday: 1.0,
					symbol: "ETH".into(),
					time: Utc::now(),
					volume_yesterday: 1.0,
				},
			);
			quotation.insert(
				"ADA",
				Quotation {
					name: "ADA".into(),
					price: 12345678.0,
					price_yesterday: 1.0,
					symbol: "ADA".into(),
					time: Utc::now(),
					volume_yesterday: 1.0,
				},
			);
			quotation.insert(
				"XRP",
				Quotation {
					name: "XRP".into(),
					price: 54321.123456789,
					price_yesterday: 1.0,
					symbol: "XRP".into(),
					time: Utc::now(),
					volume_yesterday: 1.0,
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
			Ok(Symbols { symbols: vec!["BTC".into(), "ETH".into(), "ADA".into(), "XRP".into()] })
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

		assert_eq!(c[1].price, 1000000);

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

		assert_eq!(c[0].price, 1000000);

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

		let c = storage.get_currencies_by_symbols(&["BTC", "ETH", "ADA", "XRP"]);

		assert_eq!(c[0].price, 1000000);
		assert_eq!(c[1].price, 1000000);
		assert_eq!(c[2].price, 12345678000000);
		assert_eq!(c[3].price, 54321123456);

		assert_eq!(c[1].name, "ETH");
	}
}
