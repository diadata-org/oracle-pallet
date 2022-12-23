use crate::dia::{DiaApi, Quotation, Symbols};
use crate::storage::{CoinInfo, CoinInfoStorage};
use log::{error, info};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::{error::Error, sync::Arc};

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

fn convert_to_coin_info(value: Quotation) -> Result<CoinInfo, Box<dyn Error + Sync + Send>> {
	let Quotation { name, symbol, price, time, volume_yesterday, .. } = value;

	let price = convert_decimal_to_u128(&price)?;
	let supply = convert_decimal_to_u128(&volume_yesterday)?;

	let coin_info = CoinInfo {
		name: name.into(),
		symbol: symbol.into(),
		price,
		last_update_timestamp: time.timestamp().unsigned_abs(),
		supply,
	};

	info!("Coin Price: {:#?}", price);
	info!("Coin Supply: {:#?}", volume_yesterday);
	info!("Coin Info : {:#?}", coin_info);

	Ok(coin_info)
}

async fn update_prices<T>(
	coins: Arc<CoinInfoStorage>,
	supported: &Option<HashSet<String>>,
	api: &T,
	rate: std::time::Duration,
) where
	T: DiaApi + Send + Sync + 'static,
{
	if let Ok(Symbols { symbols }) = api.get_symbols().await {
		info!("No. of currencies to retrieve : {}", symbols.len());

		let mut currencies = vec![];

		for s in symbols
			.iter()
			.filter(|x| supported.as_ref().map(|set| set.contains(x.as_str())).unwrap_or(true))
		{
			match api.get_quotation(s).await.and_then(convert_to_coin_info) {
				Ok(coin_info) => {
					currencies.push(coin_info);
				},
				Err(err) => {
					error!("Error while retrieving quotation for {}: {}", s, err)
				},
			}
			tokio::time::delay_for(rate).await;
		}
		coins.replace_currencies_by_symbols(currencies);
		info!("Currencies Updated");
	}
}
#[derive(Debug)]
pub enum ConvertingError {
	DecimalTooLarge,
}

impl Display for ConvertingError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			ConvertingError::DecimalTooLarge => write!(f, "Decimal given is too large"),
		}
	}
}

impl Error for ConvertingError {}

fn convert_decimal_to_u128(input: &Decimal) -> Result<u128, ConvertingError> {
	let fract = (input.fract() * Decimal::from(1_000_000_000_000_u128))
		.to_u128()
		.ok_or(ConvertingError::DecimalTooLarge)?;
	let trunc = (input.trunc() * Decimal::from(1_000_000_000_000_u128))
		.to_u128()
		.ok_or(ConvertingError::DecimalTooLarge)?;

	Ok(trunc.saturating_add(fract))
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
		let all_currencies = None;
		update_prices(coins, &all_currencies, &mock_api, std::time::Duration::from_secs(1)).await;

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

		assert_eq!(c[0].price, 1000000000000);

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

	#[tokio::test]
	async fn test_convert_result() {
		let mock_api = MockDia::new();
		let storage = Arc::new(CoinInfoStorage::default());
		let coins = Arc::clone(&storage);
		let all_currencies = None;

		update_prices(coins, &all_currencies, &mock_api, std::time::Duration::from_secs(1)).await;

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
