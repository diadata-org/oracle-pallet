use crate::handlers::currencies;
use crate::storage::CoinInfoStorage;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;

mod handlers;
mod price_updater;
mod storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	
	pretty_env_logger::init();

	let storage = Arc::new(CoinInfoStorage::default());
	let data = web::Data::from(storage.clone());

	// TODO: time interval should be taken from a config of some kind
	price_updater::run_update_prices_loop(storage, std::time::Duration::from_secs(1)).await;

	HttpServer::new(move || App::new().app_data(data.clone()).service(currencies))
		.bind("0.0.0.0:8080")?
		.run()
		.await
}
