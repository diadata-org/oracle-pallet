#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]
pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrappers.
/// We can use from supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// the types with this pallet-specific identifier.
pub mod crypto {
	use sp_core::crypto::KeyTypeId;
	pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"dia!");
	use frame_support::sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};
	use sp_core::sr25519::Signature as Sr25519Signature;
	app_crypto!(sr25519, KEY_TYPE);

	pub struct DiaAuthId;

	// implemented for runtime
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for DiaAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for DiaAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

#[frame_support::pallet]
pub mod pallet {
	use codec::{Decode, Encode};
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		sp_runtime::offchain,
		sp_std,
		sp_std::{vec, vec::Vec},
	};
	use frame_system::{
		offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer},
		pallet_prelude::*,
	};
	use serde::{Deserialize, Deserializer};

	const BATCHING_ENDPOINT_FALLBACK: [u8; 22] = *b"http://localhost:8080/";

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The overarching dispatch call type.
		type Call: From<Call<Self>>;
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[derive(
		Encode, Decode, scale_info::TypeInfo, Debug, Clone, PartialEq, Eq, Default, Deserialize,
	)]
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

	#[pallet::storage]
	#[pallet::getter(fn batching_api)]
	pub type BatchingApi<T: Config> = StorageValue<_, Vec<u8>>;

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

		/// Batching Api Endpoint not set.
		NoBatchingApiEndPoint,

		/// Failed Deserializing to str
		DeserializeError,

		/// Sending Http request to Batching Server Failed
		HttpRequestSendFailed,

		/// Http request to Batching Server Failed
		HttpRequestFailed,

		/// Failed to send signed Transaction
		FailedSignedTransaction,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(_n: T::BlockNumber) {
			match Self::update_prices() {
				Ok(_) => log::info!("Updated Prices"),
				Err(_) => log::error!("Failed to Update Prices"),
			}
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
		fn update_prices() -> Result<(), Error<T>> {
			// Expected contract for the API with the server is supported currencies in URL path and
			// json encoded Vec<CoinInfo> as a result from the server
			let supported_currencies: Vec<u8> = <SupportedCurrencies<T>>::iter_keys()
				.map(|mut c| {
					c.extend(b",");
					c
				})
				.flatten()
				.collect::<Vec<u8>>();

			let mut api = Self::batching_api()
				.ok_or(<Error<T>>::NoBatchingApiEndPoint) // Error Redundant but Explains Error Reason
				.unwrap_or(BATCHING_ENDPOINT_FALLBACK.to_vec());

			let request = if supported_currencies.len() < (u16::MAX as usize) {
				api.extend(supported_currencies);
				let api = sp_std::str::from_utf8(&api).map_err(|_| <Error<T>>::DeserializeError)?;
				offchain::http::Request::get(api)
			} else {
				let api = sp_std::str::from_utf8(&api).map_err(|_| <Error<T>>::DeserializeError)?;
				offchain::http::Request::post(api, vec![&supported_currencies[..]])
			};

			let pending = request.send().map_err(|_| <Error<T>>::HttpRequestSendFailed)?;
			let response = pending.wait().map_err(|_| <Error<T>>::HttpRequestFailed)?;
			let body = response.body().collect::<Vec<u8>>();

			let prices: Vec<CoinInfo> =
				serde_json::from_slice(&body).map_err(|_| <Error<T>>::DeserializeError)?;

			let prices: Vec<(Vec<u8>, CoinInfo)> =
				prices.into_iter().map(|p| (p.name.clone(), p)).collect();

			let signer = Signer::<T, T::AuthorityId>::any_account();

			signer
				.send_signed_transaction(|_| Call::<T>::set_updated_coin_infos {
					// `prices` are not `move`d because of Fn(_)
					// `prices` would have `move`d if FnOnce(_) was in signature
					// Hence the redundant clone.
					coin_infos: prices.clone(),
				})
				.ok_or(<Error<T>>::FailedSignedTransaction)?
				.1
				.map_err(|_| <Error<T>>::FailedSignedTransaction)?;

			Ok(())
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
		pub fn authorize_account(
			origin: OriginFor<T>,
			_account_id: T::AccountId,
		) -> DispatchResult {
			Pallet::<T>::check_origin_rights(origin)?;
			todo!("Should check if the origin account is authorized and if it's ok, add given account_id to the authorized set")
		}

		#[pallet::weight(10_000)]
		pub fn deauthorize_account(
			origin: OriginFor<T>,
			_account_id: T::AccountId,
		) -> DispatchResult {
			Pallet::<T>::check_origin_rights(origin)?;
			// The origin account can't deauthorize itself
			todo!("Should check if the origin account is authorized and if it's ok, should remove given account_id from the authorized set")
		}

		#[pallet::weight(10_000)]
		pub fn set_updated_coin_infos(
			origin: OriginFor<T>,
			_coin_infos: Vec<(Vec<u8>, CoinInfo)>,
		) -> DispatchResult {
			Pallet::<T>::check_origin_rights(origin)?;
			todo!("Should check authorization and after that update storage and emit event")
		}
	}
}
