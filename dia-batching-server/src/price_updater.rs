use crate::dia::{DiaApi, Quotation};
use crate::storage::{CoinInfo, CoinInfoStorage};
use crate::AssetSpecifier;
use log::{error, info};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::{error::Error, sync::Arc};

pub async fn run_update_prices_loop<T>(
	storage: Arc<CoinInfoStorage>,
	maybe_supported_currencies: Option<HashSet<AssetSpecifier>>,
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

			update_prices(coins, &maybe_supported_currencies, &api, rate).await;

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
	maybe_supported_currencies: &Option<HashSet<AssetSpecifier>>,
	api: &T,
	rate: std::time::Duration,
) where
	T: DiaApi + Send + Sync + 'static,
{
	if let Ok(quotable_assets) = api.get_quotable_assets().await {
		info!("No. of quotable assets to retrieve : {}", quotable_assets.len());

		let mut currencies = vec![];

		for quotable_asset in quotable_assets {
			let asset = AssetSpecifier {
				blockchain: quotable_asset.asset.blockchain.clone(),
				symbol: quotable_asset.asset.symbol.clone(),
			};

			if maybe_supported_currencies
				.as_ref()
				.map_or(true, |supported| supported.contains(&asset))
			{
				match api.get_quotation(&quotable_asset).await.and_then(convert_to_coin_info) {
					Ok(coin_info) => {
						currencies.push(coin_info);
					},
					Err(err) => {
						error!("Error while retrieving quotation for {:?}: {}", quotable_asset, err)
					},
				}
				tokio::time::delay_for(rate).await;
			}
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
	use crate::dia::{Asset, QuotedAsset};
	use std::{collections::HashMap, error::Error, sync::Arc};

	use async_trait::async_trait;
	use chrono::Utc;
	use rust_decimal_macros::dec;

	use super::*;

	struct MockDia {
		quotation: HashMap<AssetSpecifier, Quotation>,
	}

	impl MockDia {
		pub fn new() -> Self {
			let mut quotation = HashMap::new();
			quotation.insert(
				AssetSpecifier { blockchain: "Bitcoin".into(), symbol: "BTC".into() },
				Quotation {
					name: "BTC".into(),
					price: dec!(1.000000000000),
					price_yesterday: dec!(1.000000000000),
					symbol: "BTC".into(),
					time: Utc::now(),
					volume_yesterday: dec!(1.000000000000),
					address: "0x0000000000000000000000000000000000000000".into(),
					blockchain: "Bitcoin".into(),
					source: "diadata.org".into(),
				},
			);
			quotation.insert(
				AssetSpecifier { blockchain: "Ethereum".into(), symbol: "ETH".into() },
				Quotation {
					name: "BTC".into(),
					price: dec!(1.000000000000),
					price_yesterday: dec!(1.000000000000),
					symbol: "BTC".into(),
					time: Utc::now(),
					volume_yesterday: dec!(1.000000000000),
					address: "0x0000000000000000000000000000000000000000".into(),
					blockchain: "Bitcoin".into(),
					source: "diadata.org".into(),
				},
			);
			quotation.insert(
				AssetSpecifier { blockchain: "Ethereum".into(), symbol: "USTD".into() },
				Quotation {
					name: "BTC".into(),
					price: dec!(1.000000000000),
					price_yesterday: dec!(1.000000000000),
					symbol: "BTC".into(),
					time: Utc::now(),
					volume_yesterday: dec!(1.000000000000),
					address: "0x0000000000000000000000000000000000000000".into(),
					blockchain: "Bitcoin".into(),
					source: "diadata.org".into(),
				},
			);
			quotation.insert(
				AssetSpecifier { blockchain: "Ethereum".into(), symbol: "USDC".into() },
				Quotation {
					name: "BTC".into(),
					price: dec!(1.000000000000),
					price_yesterday: dec!(1.000000000000),
					symbol: "BTC".into(),
					time: Utc::now(),
					volume_yesterday: dec!(1.000000000000),
					address: "0x0000000000000000000000000000000000000000".into(),
					blockchain: "Bitcoin".into(),
					source: "diadata.org".into(),
				},
			);
			Self { quotation }
		}
	}

	#[async_trait]
	impl DiaApi for MockDia {
		async fn get_quotation(
			&self,
			asset: &QuotedAsset,
		) -> Result<Quotation, Box<dyn Error + Send + Sync>> {
			let QuotedAsset { asset, volume: _ } = asset;
			let asset = AssetSpecifier {
				blockchain: asset.blockchain.clone(),
				symbol: asset.symbol.clone(),
			};
			Ok(self.quotation.get(&asset).ok_or("Error Finding Quotation".to_string())?.clone())
		}

		async fn get_quotable_assets(
			&self,
		) -> Result<Vec<QuotedAsset>, Box<dyn Error + Send + Sync>> {
			Ok(vec![
				QuotedAsset {
					asset: Asset {
						symbol: "BTC".into(),
						name: "Bitcoin".into(),
						address: "0x0000000000000000000000000000000000000000".into(),
						decimals: 8,
						blockchain: "Bitcoin".into(),
					},
					volume: Decimal::new(3818975389095178, 6),
				},
				QuotedAsset {
					asset: Asset {
						symbol: "ETH".into(),
						name: "Ether".into(),
						address: "0x0000000000000000000000000000000000000000".into(),
						decimals: 18,
						blockchain: "Ethereum".into(),
					},
					volume: Decimal::new(791232743889491, 6),
				},
				QuotedAsset {
					asset: Asset {
						symbol: "USDT".into(),
						name: "Tether USD".into(),
						address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".into(),
						decimals: 6,
						blockchain: "Ethereum".into(),
					},
					volume: Decimal::new(294107237463418, 6),
				},
				QuotedAsset {
					asset: Asset {
						symbol: "USDC".into(),
						name: "USD Coin".into(),
						address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".into(),
						decimals: 6,
						blockchain: "Ethereum".into(),
					},
					volume: Decimal::new(205584209531937, 6),
				},
			])
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
