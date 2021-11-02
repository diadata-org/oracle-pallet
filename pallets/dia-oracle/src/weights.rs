#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::sp_std::marker::PhantomData;
use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};

/// Weight functions needed for pallet_quadratic_funding.
pub trait WeightInfo {
	fn add_currency() -> Weight;
	fn remove_currency() -> Weight;
	fn authorize_account() -> Weight;
	fn deauthorize_account() -> Weight;
	fn set_updated_coin_infos() -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn add_currency() -> Weight {
		(52_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn remove_currency() -> Weight {
		(52_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn authorize_account() -> Weight {
		(52_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn deauthorize_account() -> Weight {
		(52_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn set_updated_coin_infos() -> Weight {
		(52_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
}
