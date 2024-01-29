use async_trait::async_trait;
use chrono::prelude::*;
use chrono::DateTime;
use graphql_client::{GraphQLQuery, Response};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::error;
use std::error::Error;
use std::string::ToString;

const QUOTABLE_ASSETS_ENDPOINT: &str = "https://api.diadata.org/v1/quotedAssets";
/// ### Quotable Assets
///
/// `GET : https://api.diadata.org/v1/quotedAssets`
///
/// Get most recent information on the blockchain/symbol names for assets.
///
/// Example:
/// https://api.diadata.org/v1/quotedAssets
///
/// Response:
/// ```ignore
/// [{
/// 	"Asset": {
/// 		"Symbol": "BTC",
/// 		"Name": "Bitcoin",
/// 		"Address": "0x0000000000000000000000000000000000000000",
/// 		"Decimals": 8,
/// 		"Blockchain": "Bitcoin"
/// 	},
/// 	"Volume": 3818975389.095178
/// }, ...]
/// ```
#[derive(Deserialize, Debug, Clone)]
pub struct QuotedAsset {
	#[serde(rename(deserialize = "Asset"))]
	pub asset: Asset,
	#[serde(rename(deserialize = "Volume"))]
	pub volume: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Asset {
	#[serde(rename(deserialize = "Symbol"))]
	pub symbol: String,
	#[serde(rename(deserialize = "Name"))]
	pub name: String,
	#[serde(rename(deserialize = "Address"))]
	pub address: String,
	#[serde(rename(deserialize = "Decimals"))]
	pub decimals: u8,
	#[serde(rename(deserialize = "Blockchain"))]
	pub blockchain: String,
}

/// Find information on how to use it here: https://docs.diadata.org/documentation/api-1/traditional-finance-data-api-endpoints
const FOREIGN_QUOTATION_ENDPOINT: &str = "https://api.diadata.org/v1/foreignQuotation/YahooFinance";

const QUOTATION_ENDPOINT: &str = "https://api.diadata.org/v1/assetQuotation";
/// ### Quotation
///
/// `GET : https://api.diadata.org/v1/assetQuotation/:blockchain/:address`
///
/// Get most recent information on the currency corresponding to a blockchain/address pair
///
/// Example:
/// https://api.diadata.org/v1/assetQuotation/Bitcoin/0x0000000000000000000000000000000000000000
///
/// Response:
/// ```ignore
/// {
/// 	"Symbol": "BTC",
/// 	"Name": "Bitcoin",
/// 	"Address": "0x0000000000000000000000000000000000000000",
/// 	"Blockchain": "Bitcoin",
/// 	"Price": 16826.489316709616,
/// 	"PriceYesterday": 16813.219221169464,
/// 	"VolumeYesterdayUSD": 3680339928.151318,
/// 	"Time": "2022-12-24T13:33:59.982Z",
/// 	"Source": "diadata.org"
/// }
/// ```
#[derive(Deserialize, Debug, Clone)]
pub struct Quotation {
	#[serde(rename(deserialize = "Symbol"))]
	pub symbol: String,
	#[serde(rename(deserialize = "Name"))]
	pub name: String,
	#[serde(rename(deserialize = "Address"))]
	pub address: Option<String>,
	#[serde(rename(deserialize = "Blockchain"))]
	pub blockchain: Option<String>,
	#[serde(rename(deserialize = "Price"))]
	pub price: Decimal,
	#[serde(rename(deserialize = "PriceYesterday"))]
	pub price_yesterday: Decimal,
	#[serde(rename(deserialize = "VolumeYesterdayUSD"))]
	pub volume_yesterday: Decimal,
	#[serde(rename(deserialize = "Time"))]
	pub time: DateTime<Utc>,
	#[serde(rename(deserialize = "Source"))]
	pub source: String,
}

impl Default for Quotation {
	fn default() -> Self {
		Self {
			symbol: Default::default(),
			name: Default::default(),
			address: Default::default(),
			blockchain: Default::default(),
			price: Default::default(),
			price_yesterday: Default::default(),
			volume_yesterday: Default::default(),
			time: Utc::now(),
			source: Default::default(),
		}
	}
}

impl Quotation {
	pub fn get_default_fiat_usd_quotation() -> Self {
		Self {
			symbol: "USD-USD".to_string(),
			name: "USD-X".to_string(),
			address: None,
			blockchain: None,
			price: Decimal::new(1, 0),
			price_yesterday: Decimal::new(1, 0),
			volume_yesterday: Decimal::new(0, 0),
			time: Utc::now(),
			source: "YahooFinance".to_string(),
		}
	}
}

#[async_trait]
pub trait DiaApi {
	async fn get_quotable_assets(
		&self,
	) -> Result<Vec<QuotedAsset>, Box<dyn error::Error + Send + Sync>>;
	async fn get_quotation(
		&self,
		_: &QuotedAsset,
	) -> Result<Quotation, Box<dyn error::Error + Sync + Send>>;
}
pub struct Dia;

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[derive(GraphQLQuery)]
#[graphql(
	schema_path = "resources/schema.graphql",
	query_path = "resources/ampe_query.graphql",
	response_derives = "Debug"
)]
pub struct AmpePriceView;

impl AmpePriceView {
	const SYMBOL: &'static str = "AMPE";
	const BLOCKCHAIN: &'static str = "AMPLITUDE";
	const URL: &'static str = "https://squid.subsquid.io/amplitude-squid/graphql";

	/// Response:
	/// ```ignore
	/// Response {
	///     data: Some(
	///         ResponseData {
	///             bundle_by_id: AmpeViewBundleById {
	///                 eth_price: 0.003482,
	///             },
	///         },
	///     ),
	///     errors: None,
	///     extensions: None,
	/// }
	/// ```
	/// Returns the value of `eth_price`, which is the price of AMPE.
	async fn get_price() -> Result<Quotation, Box<dyn Error + Send + Sync>> {
		let request_body = AmpePriceView::build_query(ampe_price_view::Variables {});

		let client = reqwest::Client::new();
		let response = client.post(Self::URL).json(&request_body).send().await?;
		let response_body: Response<ampe_price_view::ResponseData> = response.json().await?;

		let response_data = response_body.data.ok_or("No price found for AMPE")?;
		let price = response_data.bundle_by_id.eth_price;

		Ok(Quotation {
			symbol: Self::SYMBOL.to_string(),
			name: "".to_string(),
			address: None,
			blockchain: Some(Self::BLOCKCHAIN.to_string()),
			price,
			price_yesterday: Default::default(),
			volume_yesterday: Default::default(),
			time: Default::default(),
			source: Self::URL.to_string(),
		})
	}
}

#[async_trait]
impl DiaApi for Dia {
	async fn get_quotation(
		&self,
		asset: &QuotedAsset,
	) -> Result<Quotation, Box<dyn error::Error + Send + Sync>> {
		let QuotedAsset { asset, volume: _ } = asset;

		let r = match asset.blockchain.to_uppercase().as_str() {
			"FIAT" => {
				if asset.symbol.to_uppercase() == "USD-USD" {
					return Ok(Quotation::get_default_fiat_usd_quotation());
				} else {
					let fiat_symbol = asset.symbol.to_uppercase();
					reqwest::get(&format!("{}/{}", FOREIGN_QUOTATION_ENDPOINT, fiat_symbol)).await?
				}
			},
			"AMPLITUDE" if asset.symbol.to_uppercase() == AmpePriceView::SYMBOL => {
				return AmpePriceView::get_price().await
			},
			_ => {
				reqwest::get(&format!(
					"{}/{}/{}",
					QUOTATION_ENDPOINT, asset.blockchain, asset.address
				))
				.await?
			},
		};

		let q: Quotation = r.json().await?;
		Ok(q)
	}

	async fn get_quotable_assets(
		&self,
	) -> Result<Vec<QuotedAsset>, Box<dyn error::Error + Sync + Send>> {
		let r = reqwest::get(QUOTABLE_ASSETS_ENDPOINT).await?;
		let assets = match r.json::<Vec<QuotedAsset>>().await {
			Ok(assets) => assets,
			Err(e) => {
				log::error!("Failed to parse quotable assets: {}", e);
				return Err(e.into());
			},
		};
		Ok(assets)
	}
}

#[cfg(test)]
mod tests {
	use crate::dia::{AmpePriceView, Asset, Dia, DiaApi, QuotedAsset};
	use rust_decimal::Decimal;

	#[tokio::test]
	async fn test_ampe_price() {
		let quoted_asset = QuotedAsset {
			asset: Asset {
				symbol: AmpePriceView::SYMBOL.to_string(),
				name: "".to_string(),
				address: "".to_string(),
				decimals: 0,
				blockchain: AmpePriceView::BLOCKCHAIN.to_string(),
			},
			volume: 0.0,
		};
		let price = Dia.get_quotation(&quoted_asset).await.expect("should return a quotation");

		assert_eq!(price.symbol, quoted_asset.asset.symbol);
		assert_eq!(price.blockchain.expect("should return ampe"), quoted_asset.asset.blockchain);
		assert!(price.price < Decimal::new(1, 0));
	}
}
