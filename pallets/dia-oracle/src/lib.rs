#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use codec::{Decode, Encode};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, sp_std::vec::Vec};
	use frame_system::{ensure_signed, pallet_prelude::*};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[derive(Encode, Decode, scale_info::TypeInfo, Debug, Clone, PartialEq, Eq, Default)]
	pub struct CoinInfo {
		pub symbol: Vec<u8>,
		pub name: Vec<u8>,
		pub supply: u64,
		pub last_update_timestamp: u64,
		pub price: u64,
	}

	// TODO: Maybe it should be moved to it's own crate
	pub trait DiaOracle {
		/// Returns the coin info by given name
		fn get_coin_info(name: Vec<u8>) -> Result<CoinInfo, DispatchError>;

		/// Returns the price by given name
		fn get_value(name: Vec<u8>) -> Result<u64, DispatchError>;
	}

	/// List of all authorized accounts
	#[pallet::storage]
	#[pallet::getter(fn authorized_accounts)]
	pub type AuthorizedAccounts<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ()>;

	/// List of all supported currencies
	#[pallet::storage]
	#[pallet::getter(fn supported_currencies)]
	pub type SupportedCurrencies<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, ()>;

	/// Map of all the coins names to their respective info and price
	#[pallet::storage]
	#[pallet::getter(fn prices_map)]
	pub type CoinInfosMap<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, CoinInfo, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event is triggered when prices are updated
		UpdatedPrices(Vec<(Vec<u8>, CoinInfo)>),
		/// Event is triggered when account is authorized
		AccountIdAuthorized(T::AccountId),
		/// Event is triggered when account is deauthorized
		AccountIdDeauthorized(T::AccountId),
		/// Event is triggered when currency is added to the list
		CurrencyAdded(Vec<u8>),
		/// Event is triggered when currency is remove from the list
		CurrencyRemoved(Vec<u8>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error is returned if no information is available about given coin
		NoCoinInfoAvailable,
		/// AccountId is not authorized
		ThisAccountIdIsNotAuthorized,
		/// User cannot deauthorized themself
		UserUnableToDeauthorizeThemself,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(_n: T::BlockNumber) {
			Self::update_prices();
		}
	}

	impl<T: Config> DiaOracle for Pallet<T> {
		fn get_coin_info(_name: Vec<u8>) -> Result<CoinInfo, DispatchError> {
			todo!("Return the coin info from the storage or return error if it's not found")
		}

		fn get_value(name: Vec<u8>) -> Result<u64, DispatchError> {
			<Pallet<T> as DiaOracle>::get_coin_info(name).map(|info| info.price)
		}
	}

	impl<T: Config> Pallet<T> {
		fn update_prices() {
			// Expected contract for the API with the server is supported currencies in URL path and
			// json encoded Vec<CoinInfo> as a result from the server
			todo!("Update prices information via Call::set_updated_coin_infos from the standalone server")
		}

		fn check_origin_rights(origin_account_id: &T::AccountId) -> DispatchResult {
			ensure!(
				<AuthorizedAccounts<T>>::contains_key(origin_account_id),
				Error::<T>::ThisAccountIdIsNotAuthorized
			);
			Ok(())
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn add_currency(origin: OriginFor<T>, currency_symbol: Vec<u8>) -> DispatchResult {
			let origin_account_id = ensure_signed(origin)?;
			Pallet::<T>::check_origin_rights(&origin_account_id)?;
			match <SupportedCurrencies<T>>::contains_key(&currency_symbol) {
				true => Ok(()),
				false => {
					Self::deposit_event(Event::<T>::CurrencyAdded(currency_symbol.clone()));
					<SupportedCurrencies<T>>::insert(currency_symbol, ());
					Ok(())
				}
			}
		}

		#[pallet::weight(10_000)]
		pub fn remove_currency(origin: OriginFor<T>, currency_symbol: Vec<u8>) -> DispatchResult {
			let origin_account_id = ensure_signed(origin)?;
			Pallet::<T>::check_origin_rights(&origin_account_id)?;
			match <SupportedCurrencies<T>>::contains_key(&currency_symbol) {
				true => {
					Self::deposit_event(Event::<T>::CurrencyRemoved(currency_symbol.clone()));
					<SupportedCurrencies<T>>::remove(currency_symbol);
					Ok(())
				}
				false => Ok(()),
			}
		}

		#[pallet::weight(10_000)]
		pub fn authorize_account(origin: OriginFor<T>, account_id: T::AccountId) -> DispatchResult {
			let origin_account_id = ensure_signed(origin)?;
			Pallet::<T>::check_origin_rights(&origin_account_id)?;
			match <AuthorizedAccounts<T>>::contains_key(&account_id) {
				true => Ok(()),
				false => {
					Self::deposit_event(Event::<T>::AccountIdAuthorized(account_id.clone()));
					<AuthorizedAccounts<T>>::insert(account_id, ());
					Ok(())
				}
			}
		}

		#[pallet::weight(10_000)]
		pub fn deauthorize_account(
			origin: OriginFor<T>,
			account_id: T::AccountId,
		) -> DispatchResult {
			let origin_account_id = ensure_signed(origin)?;
			Pallet::<T>::check_origin_rights(&origin_account_id)?;
			ensure!(account_id != origin_account_id, Error::<T>::UserUnableToDeauthorizeThemself);
			match <AuthorizedAccounts<T>>::contains_key(&account_id) {
				true => {
					Self::deposit_event(Event::<T>::AccountIdDeauthorized(account_id.clone()));
					<AuthorizedAccounts<T>>::remove(account_id);
					Ok(())
				}
				false => Ok(()),
			}
		}

		#[pallet::weight(10_000)]
		pub fn set_updated_coin_infos(
			origin: OriginFor<T>,
			coin_infos: Vec<(Vec<u8>, CoinInfo)>,
		) -> DispatchResult {
			let origin_account_id = ensure_signed(origin)?;
			Pallet::<T>::check_origin_rights(&origin_account_id)?;
			Self::deposit_event(Event::<T>::UpdatedPrices(coin_infos.clone()));
			for (v, c) in coin_infos.into_iter().map(|(x, y)| (x, y)) {
				<CoinInfosMap<T>>::insert(v, c);
			}
			Ok(())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate as dia_oracle;
	use frame_support::{parameter_types};
	use frame_system as system;
	use sp_core::H256;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
	};

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;

	// Configure a mock runtime to test the pallet.
	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			DOracle: dia_oracle::{Pallet, Call, Storage, Event<T>},
		}
	);

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const SS58Prefix: u8 = 42;
	}

	impl system::Config for Test {
		type BaseCallFilter = frame_support::traits::Everything;
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type Origin = Origin;
		type Call = Call;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = Event;
		type BlockHashCount = BlockHashCount;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = SS58Prefix;
		type OnSetCode = ();
	}

	impl dia_oracle::Config for Test {
		type Event = Event;
	}

	// Build genesis storage according to the mock runtime.
	pub fn new_test_ext() -> sp_io::TestExternalities {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn add_currency_should_work() {
		new_test_ext().execute_with(|| {
			<AuthorizedAccounts<Test>>::insert(1, ());

			let _test1 = DOracle::add_currency(Origin::signed(1), vec![1]);
			let _test2 = DOracle::add_currency(Origin::signed(1), vec![2]);
			let _test3 = DOracle::add_currency(Origin::signed(1), vec![3]);
			
			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![1]), true);
			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![2]), true);
			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![3]), true);
			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![4]), false);
		})
	}

	#[test]
	fn remove_currency_should_work() {
		new_test_ext().execute_with(|| {
			<AuthorizedAccounts<Test>>::insert(1, ());

			let _test1 = DOracle::add_currency(Origin::signed(1), vec![1]);
			let _test2 = DOracle::add_currency(Origin::signed(1), vec![2]);
			let _test3 = DOracle::remove_currency(Origin::signed(1), vec![2]);
			
			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![1]), true);
			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![2]), false);
		})
	}

	#[test]
	fn authorize_account_should_work() {
		new_test_ext().execute_with(|| {
			<AuthorizedAccounts<Test>>::insert(1, ());

			let _test1 = DOracle::authorize_account(Origin::signed(1), 2);
			let _test2 = DOracle::authorize_account(Origin::signed(1), 3);
			let _test3 = DOracle::authorize_account(Origin::signed(1), 4);
			
			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(2), true);
			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(3), true);
			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(4), true);
		})
	}

	#[test]
	fn deauthorize_account_should_work_without_deauthorizing_themself() {
		new_test_ext().execute_with(|| {
			<AuthorizedAccounts<Test>>::insert(1, ());
			<AuthorizedAccounts<Test>>::insert(2, ());
			<AuthorizedAccounts<Test>>::insert(3, ());

			let _test1 = DOracle::authorize_account(Origin::signed(1), 1);
			let _test2 = DOracle::authorize_account(Origin::signed(2), 2);
			let _test3 = DOracle::authorize_account(Origin::signed(3), 3);
			let _test4 = DOracle::deauthorize_account(Origin::signed(3), 1);
			let _test5 = DOracle::deauthorize_account(Origin::signed(3), 2);
			
			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(1), false);
			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(2), false);
			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(3), true);
		})
	}

	#[test]
	fn deauthorize_account_should_not_work_ny_deauthorizing_themself() {
		new_test_ext().execute_with(|| {
			<AuthorizedAccounts<Test>>::insert(1, ());
			<AuthorizedAccounts<Test>>::insert(2, ());

			let _test1 = DOracle::authorize_account(Origin::signed(1), 1);
			let _test2 = DOracle::authorize_account(Origin::signed(1), 2);
			let _test3 = DOracle::deauthorize_account(Origin::signed(2), 2);
			let _test4 = DOracle::deauthorize_account(Origin::signed(2), 1);
			
			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(1), false);
			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(2), true);
		})
	}

	#[test]
	fn set_updated_coin_infos_should_work() {
		new_test_ext().execute_with(|| {
			<AuthorizedAccounts<Test>>::insert(1, ());

			let example_info: CoinInfo = CoinInfo {
				symbol: vec![1],
				name: vec![1],
				supply: 9,
				last_update_timestamp: 9,
				price: 9,
			};
			let coin_infos = vec![(vec![1, 2, 3], CoinInfo::default()), (vec![2, 2, 2], example_info.clone())];
			let _test1 = DOracle::set_updated_coin_infos(Origin::signed(1), coin_infos);
			
			assert_eq!(<CoinInfosMap<Test>>::contains_key(vec![1, 2, 3]), true);
			assert_eq!(<CoinInfosMap<Test>>::contains_key(vec![2, 2, 2]), true);
			assert_eq!(<CoinInfosMap<Test>>::get(vec![2, 2, 2]), example_info);
			assert_eq!(<CoinInfosMap<Test>>::get(vec![1, 2, 3]), CoinInfo::default());
		})
	}

	#[test]
	fn check_origin_right_shoud_work() {
		new_test_ext().execute_with(|| {
			<AuthorizedAccounts<Test>>::insert(1, ());
			<AuthorizedAccounts<Test>>::insert(2, ());

			let _test1 = DOracle::add_currency(Origin::signed(1), vec![1]);
			let _test2 = DOracle::add_currency(Origin::signed(2), vec![2]);
			
			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![1]), true);
			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![2]), true);
		})
	}
}

