use crate::dia::Dia;
use crate::handlers::{currencies_get, currencies_post};
use crate::storage::CoinInfoStorage;

use actix_web::{web, App, HttpServer};
use std::sync::Arc;

mod dia;
mod handlers;
mod price_updater;
mod storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	pretty_env_logger::init();

	let storage = Arc::new(CoinInfoStorage::default());
	let data = web::Data::from(storage.clone());

	// TODO: time interval should be taken from a config of some kind
	let _ = tokio::join!(
		price_updater::run_update_prices_loop(
			storage,
			std::time::Duration::from_micros(1),
			std::time::Duration::from_secs(60),
			Dia,
		),
		HttpServer::new(move || {
			App::new()
				.app_data(data.clone())
				.service(currencies_get)
				.service(currencies_post)
		})
		.on_connect(|_, _| println!("Serving Request"))
		.bind("0.0.0.0:8070")?
		.run()
	);
	Ok(())
}
