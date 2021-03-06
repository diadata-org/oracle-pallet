#![cfg_attr(not(feature = "std"), no_std)]

pub use dia_oracle::{CoinInfo, PriceInfo};
use frame_support::sp_std::vec::Vec;
use sp_runtime::DispatchError;

sp_api::decl_runtime_apis! {
	pub trait DiaOracleApi{
		fn get_coin_info(name:Vec<u8>) -> Result<CoinInfo, DispatchError>;
		fn get_value(name:Vec<u8>) -> Result<PriceInfo,DispatchError>;
	}
}
