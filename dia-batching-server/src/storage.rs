use arc_swap::ArcSwap;
use std::collections::HashMap;
use smol_str::SmolStr;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CoinInfo {
    pub symbol: SmolStr,
    pub name: SmolStr,
    pub supply: u64,
    pub last_update_timestamp: u64,
    pub price: u64
}

#[derive(Debug, Default)]
pub struct CoinInfoStorage {
    currencies_by_symbol: ArcSwap<HashMap<SmolStr, CoinInfo>>
}

impl CoinInfoStorage {
    pub fn get_currencies_by_symbols<T: AsRef<str>>(&self, symbols: &[T]) -> Vec<CoinInfo> {
        let reference = self.currencies_by_symbol.load();
        symbols.iter()
            .filter_map(|key| reference.get(key.as_ref()))
            .cloned()
            .collect()
    }

    #[allow(dead_code)]
    pub fn replace_currencies_by_symbols(&self, currencies: Vec<CoinInfo>) {
        let map_to_replace_with = currencies.into_iter()
            .map(|x| (x.symbol.clone(), x))
            .collect();

        self.currencies_by_symbol.store(Arc::new(map_to_replace_with));
    }
}
