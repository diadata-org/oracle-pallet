use codec::{Decode, Encode};
use frame_support::{sp_runtime::DispatchError, sp_std::vec::Vec};
use serde::{Deserialize, Deserializer, Serialize};

#[cfg(feature = "std")]
use serde::Serializer;

// TODO: Maybe it should be moved to it's own crate
pub trait DiaOracle {
	/// Returns the coin info by given name
	fn get_coin_info(blockchain: Vec<u8>, symbol: Vec<u8>) -> Result<CoinInfo, DispatchError>;

	/// Returns the price by given name
	fn get_value(blockchain: Vec<u8>, symbol: Vec<u8>) -> Result<PriceInfo, DispatchError>;
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
	#[serde(deserialize_with = "de_string_to_bytes")]
	pub blockchain: Vec<u8>,
	pub supply: u128,
	pub last_update_timestamp: u64,
	pub price: u128,
}

pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(de)?;
	Ok(s.as_bytes().to_vec())
}

#[derive(Encode, Decode, scale_info::TypeInfo, Debug, Deserialize, Serialize)]
pub struct AssetId {
	pub blockchain: Vec<u8>,
	pub symbol: Vec<u8>,
}

impl AssetId {
	pub fn new(blockchain: Vec<u8>, symbol: Vec<u8>) -> Self {
		AssetId { blockchain, symbol }
	}
}

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct PriceInfo {
	pub value: u128,
}

#[cfg(feature = "std")]
impl Serialize for PriceInfo {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.value.to_string())
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for PriceInfo {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		s.parse::<u128>()
			.map(|x| PriceInfo { value: x })
			.map_err(|_| serde::de::Error::custom("Parse from str failed"))
	}
}
