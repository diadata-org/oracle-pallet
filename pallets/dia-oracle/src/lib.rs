#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use codec::{Encode, Decode};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, sp_std::vec::Vec};
	use frame_system::pallet_prelude::*;

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
		pub price: u64
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
		CurrencyRemoved(Vec<u8>)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error is returned if no information is available about given coin
		NoCoinInfoAvailable,
		/// AccountId is not authorized
		ThisAccountIdIsNotAuthorized
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

		fn check_origin_rights(_origin: OriginFor<T>) -> DispatchResult {
			todo!("Should return \"not authorized error\" when not authorized origin is given")
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn add_currency(origin: OriginFor<T>, _currency_symbol: Vec<u8>) -> DispatchResult {
			Pallet::<T>::check_origin_rights(origin)?;
			todo!("Should check if the origin account is authorized and if it's ok, add given currency to the set")
		}

		#[pallet::weight(10_000)]
		pub fn remove_currency(origin: OriginFor<T>, _currency_symbol: Vec<u8>) -> DispatchResult {
			Pallet::<T>::check_origin_rights(origin)?;
			todo!("Should check if the origin account is authorized and if it's ok, remove given currency from the set")
		}

		#[pallet::weight(10_000)]
		pub fn authorize_account(origin: OriginFor<T>, _account_id: T::AccountId) -> DispatchResult {
			Pallet::<T>::check_origin_rights(origin)?;
			todo!("Should check if the origin account is authorized and if it's ok, add given account_id to the authorized set")
		}


		#[pallet::weight(10_000)]
		pub fn deauthorize_account(origin: OriginFor<T>, _account_id: T::AccountId) -> DispatchResult {
			Pallet::<T>::check_origin_rights(origin)?;
			// The origin account can't deauthorize itself
			todo!("Should check if the origin account is authorized and if it's ok, should remove given account_id from the authorized set")
		}

		#[pallet::weight(10_000)]
		pub fn set_updated_coin_infos(origin: OriginFor<T>, _coin_infos: Vec<(Vec<u8>, CoinInfo)>) -> DispatchResult {
			Pallet::<T>::check_origin_rights(origin)?;
			todo!("Should check authorization and after that update storage and emit event")
		}
	}
}
