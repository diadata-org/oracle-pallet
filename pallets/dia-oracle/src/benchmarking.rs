//! Benchmarking setup for dia-oracle
//!
use super::*;

#[allow(unused)]
use crate::Pallet as DiaOracle;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	add_currency {
		let caller: T::AccountId = whitelisted_caller();

	}: _(RawOrigin::Signed(caller), vec![1,2,3])

	remove_currency {
		let caller: T::AccountId = whitelisted_caller();

	} : _(RawOrigin::Signed(caller), vec![1,2,3])

	// authorize_account {
	// 	let caller: T::AccountId = whitelisted_caller();

	// } : _(RawOrigin::Signed(caller), T::AccountId)


}

impl_benchmark_test_suite!(DiaOracle, crate::mock::new_test_ext(), crate::mock::Test,);
