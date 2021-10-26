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
	let dia_api = Arc::clone(&api);
	println!("Hello World");
	tokio::spawn(async move {
		loop {
			let time_elapsed = std::time::Instant::now();
			if let Ok(Symbols { symbols }) = dia_api.get_symbols().await {
				info!("No. of currencies to retrieve : {}", symbols.len());

				let mut currencies = vec![];

				for s in &symbols {
					if let Ok(Quotation { name, symbol, price, time, volume_yesterday, .. }) =
						dia_api.get_quotation(s).await
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

#[cfg(test)]
mod tests {
	use std::{collections::HashMap, error::Error, sync::Arc};

	use async_trait::async_trait;

	use super::*;

	struct MockDia<'a> {
		quotation: HashMap<&'a str, Quotation>,
	}

	impl<'a> MockDia<'a> {
		pub fn new() -> Self {
			let mut quotation = HashMap::new();
			quotation.insert("BTC", Quotation::default());
			quotation.insert("ETH", Quotation::default());

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
			Ok(self.quotation.get(symbol).ok_or("She".to_string())?.clone())
		}

		async fn get_symbols(&self) -> Result<Self::Symbols, Box<dyn Error + Send + Sync>> {
			Ok(Symbols { symbols: vec!["BTC".into(), "ETC".into()] })
		}
	}
	#[tokio::test]
	async fn test_run_update_prices_loop() {
		let mock_api = MockDia::new();
		let storage = Arc::new(CoinInfoStorage::default());

		run_update_prices_loop(
			storage,
			std::time::Duration::from_secs(1),
			std::time::Duration::from_secs(60),
			mock_api,
		)
		.await;
	}
}
