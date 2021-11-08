//! Autogenerated weights for `dia_oracle`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2021-11-03, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// target/debug/node-template
// benchmark
// --chain
// dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// dia-oracle
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --raw
// --output
// ./


#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight,constants::RocksDbWeight}};
use frame_support::sp_std::marker::PhantomData;

/// Weight functions for `dia_oracle`.
/// 
/// 
pub trait WeightInfo{
	fn add_currency() -> Weight ;
	fn remove_currency() -> Weight ;
	fn authorize_account() -> Weight ;
	fn authorize_account_signed() -> Weight ;
	fn deauthorize_account() -> Weight ;
	fn deauthorize_account_signed() -> Weight ;
	fn set_updated_coin_infos() -> Weight; 
}
pub struct DiaWeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for DiaWeightInfo<T> {
	// Storage: DiaOracle AuthorizedAccounts (r:1 w:0)
	// Storage: DiaOracle SupportedCurrencies (r:1 w:1)
	fn add_currency() -> Weight {
		(1_494_649_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: DiaOracle AuthorizedAccounts (r:1 w:0)
	// Storage: DiaOracle SupportedCurrencies (r:1 w:0)
	fn remove_currency() -> Weight {
		(542_550_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
	}
	// Storage: DiaOracle AuthorizedAccounts (r:1 w:1)
	fn authorize_account() -> Weight {
		(1_241_248_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: DiaOracle AuthorizedAccounts (r:2 w:1)
	fn authorize_account_signed() -> Weight {
		(1_525_600_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: DiaOracle AuthorizedAccounts (r:1 w:0)
	fn deauthorize_account() -> Weight {
		(276_664_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
	}
	// Storage: DiaOracle AuthorizedAccounts (r:2 w:1)
	fn deauthorize_account_signed() -> Weight {
		(1_513_398_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: DiaOracle AuthorizedAccounts (r:1 w:0)
	// Storage: DiaOracle CoinInfosMap (r:0 w:1)
	fn set_updated_coin_infos() -> Weight {
		(1_152_148_682_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}


impl WeightInfo for () {
	// Storage: DiaOracle AuthorizedAccounts (r:1 w:0)
	// Storage: DiaOracle SupportedCurrencies (r:1 w:1)
	fn add_currency() -> Weight {
		(1_494_649_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: DiaOracle AuthorizedAccounts (r:1 w:0)
	// Storage: DiaOracle SupportedCurrencies (r:1 w:0)
	fn remove_currency() -> Weight {
		(542_550_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
	}
	// Storage: DiaOracle AuthorizedAccounts (r:1 w:1)
	fn authorize_account() -> Weight {
		(1_241_248_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: DiaOracle AuthorizedAccounts (r:2 w:1)
	fn authorize_account_signed() -> Weight {
		(1_525_600_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: DiaOracle AuthorizedAccounts (r:1 w:0)
	fn deauthorize_account() -> Weight {
		(276_664_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
	}
	// Storage: DiaOracle AuthorizedAccounts (r:2 w:1)
	fn deauthorize_account_signed() -> Weight {
		(1_513_398_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: DiaOracle AuthorizedAccounts (r:1 w:0)
	// Storage: DiaOracle CoinInfosMap (r:0 w:1)
	fn set_updated_coin_infos() -> Weight {
		(1_152_148_682_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
}
