use crate::dia::{DiaApi, Quotation, Symbols};
use crate::storage::{CoinInfo, CoinInfoStorage};

use log::{error, info};
use std::sync::Arc;

pub async fn run_update_prices_loop<T>(
	storage: Arc<CoinInfoStorage>,
	rate: std::time::Duration,
	duration: std::time::Duration,
	api: T,
) where
	T: DiaApi<Symbols = Symbols, Quotation = Quotation> + Send + Sync + 'static,
{
	let coins = Arc::clone(&storage);
	let api = Arc::new(api);
	tokio::spawn(async move {
		loop {
			let time_elapsed = std::time::Instant::now();

			let coins = Arc::clone(&coins);
			let dia_api = Arc::clone(&api);

			update_prices(coins, dia_api, rate).await;

			tokio::time::delay_for(duration.saturating_sub(time_elapsed.elapsed())).await;
		}
	})
	.await
	.unwrap();
}

async fn update_prices<T>(coins: Arc<CoinInfoStorage>, api: Arc<T>, rate: std::time::Duration)
where
	T: DiaApi<Symbols = Symbols, Quotation = Quotation> + Send + Sync + 'static,
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
			tokio::time::delay_for(rate).await;
		}
		coins.replace_currencies_by_symbols(currencies);
		info!("Currencies Updated");
	}
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

#[cfg(test)]
mod tests {
	use std::{collections::HashMap, error::Error, sync::Arc};

	use async_trait::async_trait;
	use chrono::Utc;

	use super::*;

	struct MockDia<'a> {
		quotation: HashMap<&'a str, Quotation>,
	}

	unsafe impl<'a> Send for MockDia<'a> {}
	unsafe impl<'a> Sync for MockDia<'a> {}

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
		type Symbols = Symbols;
		type Quotation = Quotation;

		async fn get_quotation(
			&self,
			symbol: &str,
		) -> Result<Self::Quotation, Box<dyn Error + Send + Sync>> {
			Ok(self.quotation.get(symbol).ok_or("Error Finding Quotation".to_string())?.clone())
		}

		async fn get_symbols(&self) -> Result<Self::Symbols, Box<dyn Error + Send + Sync>> {
			Ok(Symbols { symbols: vec!["BTC".into(), "ETH".into()] })
		}
	}
	#[tokio::test]
	async fn test_update_prices() {
		let mock_api = MockDia::new();
		let api = Arc::new(mock_api);
		let storage = Arc::new(CoinInfoStorage::default());
		let coins = Arc::clone(&storage);

		update_prices(coins, api, std::time::Duration::from_secs(1)).await;

		let c = storage.get_currencies_by_symbols(&["BTC", "ETH"]);

		assert_eq!(2, c.len());

		assert_eq!(c[1].price, 1);

		assert_eq!(c[1].name, "ETH");
	}

	#[tokio::test]
	async fn test_update_prices_non_existent() {
		let mock_api = MockDia::new();
		let api = Arc::new(mock_api);
		let storage = Arc::new(CoinInfoStorage::default());
		let coins = Arc::clone(&storage);

		update_prices(coins, api, std::time::Duration::from_secs(1)).await;

		let c = storage.get_currencies_by_symbols(&["BTCCash", "ETHCase"]);

		assert_eq!(0, c.len());
	}

	#[tokio::test]
	async fn test_update_prices_one_available() {
		let mock_api = MockDia::new();
		let api = Arc::new(mock_api);
		let storage = Arc::new(CoinInfoStorage::default());
		let coins = Arc::clone(&storage);

		update_prices(coins, api, std::time::Duration::from_secs(1)).await;

		let c = storage.get_currencies_by_symbols(&["BTC", "ETHCase"]);

		assert_eq!(1, c.len());

		assert_eq!(c[0].price, 1);

		assert_eq!(c[0].name, "BTC");
	}
}
