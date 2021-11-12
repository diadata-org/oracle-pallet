use codec::{Decode, Encode};
use frame_support::{sp_runtime::DispatchError, sp_std::vec::Vec};
use serde::{Deserialize, Deserializer, Serialize};

// TODO: Maybe it should be moved to it's own crate
pub trait DiaOracle {
	/// Returns the coin info by given name
	fn get_coin_info(name: Vec<u8>) -> Result<CoinInfo, DispatchError>;

	/// Returns the price by given name
	fn get_value(name: Vec<u8>) -> Result<u64, DispatchError>;
}

#[derive(
	Encode,
	Decode,
	scale_info::TypeInfo,
	Debug,
	Clone,
	PartialEq,
	Eq,
	Default,
	Deserialize,
	Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct CoinInfo {
	#[serde(deserialize_with = "de_string_to_bytes")]
	pub symbol: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	pub name: Vec<u8>,
	pub supply: u64,
	pub last_update_timestamp: u64,
	pub price: u64,
}
pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(de)?;
	Ok(s.as_bytes().to_vec())
}
