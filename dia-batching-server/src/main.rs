use std::error::Error;
use crate::dia::Dia;
use crate::handlers::{currencies_get, currencies_post};
use crate::storage::CoinInfoStorage;

use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use crate::args::DiaApiArgs;
use structopt::StructOpt;

mod dia;
mod handlers;
mod price_updater;
mod storage;
mod args;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
	pretty_env_logger::init();

	let args: DiaApiArgs = DiaApiArgs::from_args();
	let storage = Arc::new(CoinInfoStorage::default());
	let data = web::Data::from(storage.clone());

	price_updater::run_update_prices_loop(
		storage,
		args.supported_currencies.filter(|x| x.len() > 0).map(|curs| curs.into_iter().collect()),
		std::time::Duration::from_millis(args.request_timeout_in_milliseconds),
		std::time::Duration::from_secs(args.iteration_timeout_in_seconds),
		Dia,
	).await?;

	HttpServer::new(move || {
		App::new()
			.app_data(data.clone())
			.service(currencies_get)
			.service(currencies_post)
	})
	.on_connect(|_, _| println!("Serving Request"))
	.bind("0.0.0.0:8070")?
	.run()
	.await?;

	Ok(())
}
