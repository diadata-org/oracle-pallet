use crate::storage::{CoinInfo, CoinInfoStorage};
use actix_web::web::Json;
use actix_web::{get, post, web};
use serde::{
	de::{self, IntoDeserializer},
	Deserialize,
};
use std::fmt;

#[get("/currencies/{symbols}")]
pub async fn currencies_get(
	web::Path(Currencies(symbols)): web::Path<Currencies>,
	storage: web::Data<CoinInfoStorage>,
) -> Json<Vec<CoinInfo>> {
	Json(storage.get_ref().get_currencies_by_symbols(&symbols))
}

#[post("/currencies")]
pub async fn currencies_post(
	web::Json(Currencies(symbols)): web::Json<Currencies>,
	storage: web::Data<CoinInfoStorage>,
) -> Json<Vec<CoinInfo>> {
	Json(storage.get_ref().get_currencies_by_symbols(&symbols))
}

#[derive(Deserialize)]
pub struct Currencies(#[serde(deserialize_with = "deserialize_commas")] Vec<String>);

pub fn deserialize_commas<'de, D, I>(deserializer: D) -> std::result::Result<Vec<I>, D::Error>
where
	D: de::Deserializer<'de>,
	I: de::DeserializeOwned,
{
	struct CommaSeparatedStringVisitor<I>(std::marker::PhantomData<I>);

	impl<'de, I> de::Visitor<'de> for CommaSeparatedStringVisitor<I>
	where
		I: de::DeserializeOwned,
	{
		type Value = Vec<I>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("A list of strings separated by commas")
		}

		fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
		where
			E: de::Error,
		{
			let mut ids = Vec::new();
			for id in v.split(",") {
				let id = I::deserialize(id.into_deserializer())?;
				ids.push(id);
			}
			Ok(ids)
		}
	}

	deserializer.deserialize_str(CommaSeparatedStringVisitor(std::marker::PhantomData::<I>))
}
#[cfg(test)]
mod tests {
	use super::*;
	use actix_web::{http, test, App};
	use std::sync::Arc;

	fn get_storage() -> Arc<CoinInfoStorage> {
		let storage = Arc::new(CoinInfoStorage::default());
		storage.replace_currencies_by_symbols(vec![
			CoinInfo { symbol: "BTC".into(), ..Default::default() },
			CoinInfo { symbol: "ETH".into(), ..Default::default() },
		]);
		storage
	}
	#[tokio::test]
	async fn test_currencies_get() {
		let storage = get_storage();

		let data = web::Data::from(storage.clone());
		let mut app =
			test::init_service(App::new().app_data(data.clone()).service(currencies_get)).await;
		let req =
			test::TestRequest::with_uri("http://localhost:8080/currencies/BTC,ETH,").to_request();

		let resp = test::call_service(&mut app, req).await;

		assert_eq!(resp.status(), http::StatusCode::OK);

		let r: Vec<CoinInfo> = test::read_body_json(resp).await;

		assert_eq!(r.len(), 2);
	}

	#[tokio::test]
	async fn test_currencies_post() {
		let storage = get_storage();
		let data = web::Data::from(storage.clone());

		let mut app =
			test::init_service(App::new().app_data(data.clone()).service(currencies_post)).await;
		let req = test::TestRequest::post()
			.uri("http://localhost:8080/currencies")
			.set_json(&"BTC,ETH")
			.to_request();

		let resp = test::call_service(&mut app, req).await;

		assert_eq!(resp.status(), http::StatusCode::OK);

		let r: Vec<CoinInfo> = test::read_body_json(resp).await;

		assert_eq!(r.len(), 2);
	}

	#[tokio::test]
	async fn test_currencies_get_empty() {
		let storage = get_storage();

		let data = web::Data::from(storage.clone());
		let mut app =
			test::init_service(App::new().app_data(data.clone()).service(currencies_get)).await;
		let req = test::TestRequest::with_uri("http://localhost:8080/currencies/").to_request();

		let resp = test::call_service(&mut app, req).await;

		assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
	}

	#[tokio::test]
	async fn test_currencies_get_non_existent() {
		let storage = get_storage();

		let data = web::Data::from(storage.clone());
		let mut app =
			test::init_service(App::new().app_data(data.clone()).service(currencies_get)).await;
		let req = test::TestRequest::with_uri("http://localhost:8080/currencies/DASH").to_request();

		let resp = test::call_service(&mut app, req).await;

		assert_eq!(resp.status(), http::StatusCode::OK);

		let r: Vec<CoinInfo> = test::read_body_json(resp).await;

		assert_eq!(r.len(), 0);
	}

	#[tokio::test]
	async fn test_currencies_post_empty() {
		let storage = get_storage();
		let data = web::Data::from(storage.clone());

		let mut app =
			test::init_service(App::new().app_data(data.clone()).service(currencies_post)).await;
		let req = test::TestRequest::post()
			.uri("http://localhost:8080/currencies")
			.set_json(&"")
			.to_request();

		let resp = test::call_service(&mut app, req).await;

		assert_eq!(resp.status(), http::StatusCode::OK);

		let r: Vec<CoinInfo> = test::read_body_json(resp).await;

		assert_eq!(r.len(), 0);
	}
	#[tokio::test]
	async fn test_currencies_post_non_existent() {
		let storage = get_storage();
		let data = web::Data::from(storage.clone());

		let mut app =
			test::init_service(App::new().app_data(data.clone()).service(currencies_post)).await;
		let req = test::TestRequest::post()
			.uri("http://localhost:8080/currencies")
			.set_json(&"DASH")
			.to_request();

		let resp = test::call_service(&mut app, req).await;

		assert_eq!(resp.status(), http::StatusCode::OK);

		let r: Vec<CoinInfo> = test::read_body_json(resp).await;

		assert_eq!(r.len(), 0);
	}

	#[tokio::test]
	async fn test_currencies_get_non_existent_plus_existing() {
		let storage = get_storage();

		let data = web::Data::from(storage.clone());
		let mut app =
			test::init_service(App::new().app_data(data.clone()).service(currencies_get)).await;
		let req =
			test::TestRequest::with_uri("http://localhost:8080/currencies/DASH,ETH").to_request();

		let resp = test::call_service(&mut app, req).await;

		assert_eq!(resp.status(), http::StatusCode::OK);

		let r: Vec<CoinInfo> = test::read_body_json(resp).await;

		assert_eq!(r.len(), 1);

		assert_eq!(r[0].symbol, smol_str::SmolStr::new_inline("ETH".into()))
	}

	#[tokio::test]
	async fn test_currencies_post_non_existent_plus_existing() {
		let storage = get_storage();
		let data = web::Data::from(storage.clone());

		let mut app =
			test::init_service(App::new().app_data(data.clone()).service(currencies_post)).await;
		let req = test::TestRequest::post()
			.uri("http://localhost:8080/currencies")
			.set_json(&"DASH,ETH")
			.to_request();

		let resp = test::call_service(&mut app, req).await;

		assert_eq!(resp.status(), http::StatusCode::OK);

		let r: Vec<CoinInfo> = test::read_body_json(resp).await;

		assert_eq!(r.len(), 1);

		assert_eq!(r[0].symbol, smol_str::SmolStr::new_inline("ETH".into()))
	}

	#[tokio::test]
	async fn test_currencies_get_empty_str() {
		let storage = get_storage();

		let data = web::Data::from(storage.clone());
		let mut app =
			test::init_service(App::new().app_data(data.clone()).service(currencies_get)).await;
		let req = test::TestRequest::with_uri("http://localhost:8080/currencies/,,,").to_request();

		let resp = test::call_service(&mut app, req).await;

		assert_eq!(resp.status(), http::StatusCode::OK);

		let r: Vec<CoinInfo> = test::read_body_json(resp).await;

		assert_eq!(r.len(), 0);
	}

	#[tokio::test]
	async fn test_currencies_get_diff_separator() {
		let storage = get_storage();

		let data = web::Data::from(storage.clone());
		let mut app =
			test::init_service(App::new().app_data(data.clone()).service(currencies_get)).await;
		let req =
			test::TestRequest::with_uri("http://localhost:8080/currencies/ABC;ASD;").to_request();

		let resp = test::call_service(&mut app, req).await;

		assert_eq!(resp.status(), http::StatusCode::OK);

		let r: Vec<CoinInfo> = test::read_body_json(resp).await;

		assert_eq!(r.len(), 0);
	}

	#[tokio::test]
	async fn test_currencies_get_special_char() {
		let storage = get_storage();

		let data = web::Data::from(storage.clone());
		let mut app =
			test::init_service(App::new().app_data(data.clone()).service(currencies_get)).await;
		let req =
			test::TestRequest::with_uri("http://localhost:8080/currencies/$COIN").to_request();

		let resp = test::call_service(&mut app, req).await;

		assert_eq!(resp.status(), http::StatusCode::OK);

		let r: Vec<CoinInfo> = test::read_body_json(resp).await;

		assert_eq!(r.len(), 0);
	}

	#[tokio::test]
	async fn test_currencies_post_empty_string() {
		let storage = get_storage();
		let data = web::Data::from(storage.clone());

		let mut app =
			test::init_service(App::new().app_data(data.clone()).service(currencies_post)).await;
		let req = test::TestRequest::post()
			.uri("http://localhost:8080/currencies")
			.set_json(&",,,")
			.to_request();

		let resp = test::call_service(&mut app, req).await;

		assert_eq!(resp.status(), http::StatusCode::OK);

		let r: Vec<CoinInfo> = test::read_body_json(resp).await;

		assert_eq!(r.len(), 0);
	}
	#[tokio::test]
	async fn test_currencies_post_special_char() {
		let storage = get_storage();
		let data = web::Data::from(storage.clone());

		let mut app =
			test::init_service(App::new().app_data(data.clone()).service(currencies_post)).await;
		let req = test::TestRequest::post()
			.uri("http://localhost:8080/currencies")
			.set_json(&"$COIN")
			.to_request();

		let resp = test::call_service(&mut app, req).await;

		assert_eq!(resp.status(), http::StatusCode::OK);

		let r: Vec<CoinInfo> = test::read_body_json(resp).await;

		assert_eq!(r.len(), 0);
	}

	#[tokio::test]
	async fn test_currencies_post_diff_sep() {
		let storage = get_storage();
		let data = web::Data::from(storage.clone());

		let mut app =
			test::init_service(App::new().app_data(data.clone()).service(currencies_post)).await;
		let req = test::TestRequest::post()
			.uri("http://localhost:8080/currencies")
			.set_json(&"ABC;AB;")
			.to_request();

		let resp = test::call_service(&mut app, req).await;

		assert_eq!(resp.status(), http::StatusCode::OK);

		let r: Vec<CoinInfo> = test::read_body_json(resp).await;

		assert_eq!(r.len(), 0);
	}
}
