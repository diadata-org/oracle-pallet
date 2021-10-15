use actix_web::{get, web};
use serde::{de::{self, IntoDeserializer}, Deserialize};
use std::fmt;
use actix_web::web::Json;
use crate::storage::{CoinInfoStorage, CoinInfo};

#[get("/currencies/{symbols}")]
pub async fn currencies(web::Path(Currencies(symbols)): web::Path<Currencies>, storage: web::Data<CoinInfoStorage>) -> Json<Vec<CoinInfo>> {
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
        where I: de::DeserializeOwned {
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
