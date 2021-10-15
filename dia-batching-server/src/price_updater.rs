use crate::storage::CoinInfoStorage;
use std::sync::Arc;

pub fn run_update_prices_loop(_storage: Arc<CoinInfoStorage>, _duration: std::time::Duration) {
    // TODO: create thread or task to update prices with specified time interval
}