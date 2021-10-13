#![cfg_attr(not(feature = "std"), no_std)]
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
		pub supply: u64,
		pub last_update_timestamp: u64,
		pub price: u64
	}

	// TODO: Maybe it should be moved to it's own crate
	pub trait DiaOracle {
		// Returns the coin info by given name
		fn get_coin_info(name: Vec<u8>) -> Result<CoinInfo, DispatchError>;
	}

	/// List of all authorized accounts
	#[pallet::storage]
	#[pallet::getter(fn authorized_accounts)]
	pub type AuthorizedAccounts<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ()>;

	/// Map of all the coins to their respective info and price
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
		AccountIdDeauthorized(T::AccountId)
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
		fn get_coin_info(name: Vec<u8>) -> Result<CoinInfo, DispatchError> {
			todo!("Return the price from the storage or return error if it's not found")
		}
	}

	impl<T: Config> Pallet<T> {
		fn update_prices() {
			todo!("Update prices information via Call::set_updated_coin_infos from the standalone server or from the DIA bulk API")
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn authorize_account(origin: OriginFor<T>, account_id: T::AccountId) -> DispatchResult {
			todo!("Should check if the origin account is authorized and if it's ok, add given account_id to the authorized set")
		}


		#[pallet::weight(10_000)]
		pub fn deauthorize_account(origin: OriginFor<T>, account_id: T::AccountId) -> DispatchResult {
			// The origin account can't deauthorize itself
			todo!("Should check if the origin account is authorized and if it's ok, should remove given account_id from the authorized set")
		}

		#[pallet::weight(10_000)]
		pub fn set_updated_coin_infos(origin: OriginFor<T>, coin_infos: Vec<(Vec<u8>, CoinInfo)>) -> DispatchResult {
			todo!("Should check authorization and after that update storage and emit event")
		}
	}
}
