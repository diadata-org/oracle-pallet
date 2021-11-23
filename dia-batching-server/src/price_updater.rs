use crate::dia::{DiaApi, Quotation, Symbols};
use crate::storage::{CoinInfo, CoinInfoStorage};

use log::{error, info};
use std::{error::Error, sync::Arc};
use std::collections::HashSet;

pub async fn run_update_prices_loop<T>(
	storage: Arc<CoinInfoStorage>,
	supported_currencies: Option<HashSet<String>>,
	rate: std::time::Duration,
	duration: std::time::Duration,
	api: T,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>>
where
	T: DiaApi + Send + Sync + 'static,
{
	let coins = Arc::clone(&storage);
	let _ = tokio::spawn(async move {
		loop {
			let time_elapsed = std::time::Instant::now();

			let coins = Arc::clone(&coins);

			update_prices(coins, &supported_currencies, &api, rate).await;

			tokio::time::delay_for(duration.saturating_sub(time_elapsed.elapsed())).await;
		}
	});

	Ok(())
}

async fn update_prices<T>(coins: Arc<CoinInfoStorage>, supported: &Option<HashSet<String>>, api: &T, rate: std::time::Duration)
where
	T: DiaApi + Send + Sync + 'static,
{
	if let Ok(Symbols { symbols }) = api.get_symbols().await {
		info!("No. of currencies to retrieve : {}", symbols.len());

		let mut currencies = vec![];

		for s in symbols.iter().filter(|x| supported.as_ref().map(|set| set.contains(x.as_str())).unwrap_or(true)) {
			match api.get_quotation(s).await {
				Ok(Quotation { name, symbol, price, time, volume_yesterday, .. }) => {
					let coin_info = CoinInfo {
						name: name.into(),
						symbol: symbol.into(),
						price: convert_f64_to_u64(price), // Converting f64 to u64
						last_update_timestamp: time.timestamp().unsigned_abs(),
						supply: convert_f64_to_u64(volume_yesterday), // Converting f64 to u64
					};

					info!("Coin Price: {:#?}", price);
					info!("Coin Supply: {:#?}", volume_yesterday);
					info!("Coin Info : {:#?}", coin_info);

					currencies.push(coin_info);
				},
				Err(err) => {
					error!("Error while retrieving quotation for {}: {}", s, err)
				}
			}
			tokio::time::delay_for(rate).await;
		}
		coins.replace_currencies_by_symbols(currencies);
		info!("Currencies Updated");
	}
}

fn convert_f64_to_u64(input: f64) -> u64 {
	let scaled = input * 1_000_000.0;
	scaled.abs().ceil() as u64
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
			Ok(Symbols { symbols: vec!["BTC".into(), "ETH".into()] })
		}
	}
	#[tokio::test]
	async fn test_update_prices() {
		let mock_api = MockDia::new();
		let storage = Arc::new(CoinInfoStorage::default());
		let coins = Arc::clone(&storage);
		let all_currencies = None;
		update_prices(coins, &all_currencies, &mock_api, std::time::Duration::from_secs(1)).await;

		let c = storage.get_currencies_by_symbols(&["BTC", "ETH"]);

		assert_eq!(2, c.len());

		assert_eq!(c[1].price, 1_000_000);

		assert_eq!(c[1].name, "ETH");
	}

	#[tokio::test]
	async fn test_update_prices_non_existent() {
		let mock_api = MockDia::new();
		let storage = Arc::new(CoinInfoStorage::default());
		let coins = Arc::clone(&storage);
		let all_currencies = None;
		update_prices(coins, &all_currencies, &mock_api, std::time::Duration::from_secs(1)).await;

		let c = storage.get_currencies_by_symbols(&["BTCCash", "ETHCase"]);

		assert_eq!(0, c.len());
	}

	#[tokio::test]
	async fn test_update_prices_one_available() {
		let mock_api = MockDia::new();
		let storage = Arc::new(CoinInfoStorage::default());
		let coins = Arc::clone(&storage);
		let all_currencies = None;
		update_prices(coins, &all_currencies, &mock_api, std::time::Duration::from_secs(1)).await;

		let c = storage.get_currencies_by_symbols(&["BTC", "ETHCase"]);

		assert_eq!(1, c.len());

		assert_eq!(c[0].price, 1_000_000);

		assert_eq!(c[0].name, "BTC");
	}

	#[tokio::test]
	async fn test_update_prices_get_nothing() {
		let mock_api = MockDia::new();
		let storage = Arc::new(CoinInfoStorage::default());
		let coins = Arc::clone(&storage);
		let all_currencies = None;
		update_prices(coins, &all_currencies, &mock_api, std::time::Duration::from_secs(1)).await;

		let c = storage.get_currencies_by_symbols::<&str>(&[]);

		assert_eq!(0, c.len());
	}

	#[tokio::test]
	async fn test_update_prices_get_integers() {
		let mock_api = MockDia::new();
		let storage = Arc::new(CoinInfoStorage::default());
		let coins = Arc::clone(&storage);
		let all_currencies = None;

		update_prices(coins, &all_currencies, &mock_api, std::time::Duration::from_secs(1)).await;

		let c = storage.get_currencies_by_symbols(&["123"]);

		assert_eq!(0, c.len());
	}
}
