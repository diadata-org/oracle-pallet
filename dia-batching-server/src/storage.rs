use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::Arc;

use crate::handlers::Currency;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinInfo {
	pub symbol: SmolStr,
	pub name: SmolStr,
	pub blockchain: SmolStr,
	pub supply: u128,
	pub last_update_timestamp: u64,
	pub price: u128,
}

#[derive(Debug, Default)]
pub struct CoinInfoStorage {
	currencies_by_blockchain_and_symbol: ArcSwap<HashMap<(SmolStr, SmolStr), CoinInfo>>,
}

impl CoinInfoStorage {
	pub fn get_currencies_by_blockchains_and_symbols(
		&self,
		blockchain_and_symbols: Vec<Currency>,
	) -> Vec<CoinInfo> {
		let reference = self.currencies_by_blockchain_and_symbol.load();
		blockchain_and_symbols
			.iter()
			.filter_map(|Currency { blockchain, symbol }| {
				reference.get(&(blockchain.into(), symbol.into()))
			})
			.cloned()
			.collect()
	}

	#[allow(dead_code)]
	pub fn replace_currencies_by_symbols(&self, currencies: Vec<CoinInfo>) {
		let map_to_replace_with = currencies
			.into_iter()
			.map(|x| ((x.blockchain.clone(), x.symbol.clone()), x))
			.collect();

		self.currencies_by_blockchain_and_symbol.store(Arc::new(map_to_replace_with));
	}
}
