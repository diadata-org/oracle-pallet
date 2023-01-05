#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]
pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod mock;

pub mod dia;
pub use dia::*;
pub mod weights;
pub use sp_std::convert::TryInto;
pub use weights::WeightInfo;

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
	use sp_std::convert::TryFrom;

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
	use super::*;

	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		sp_runtime::offchain,
		sp_std,
		sp_std::{vec, vec::Vec},
	};
	use frame_system::{
		ensure_root, ensure_signed,
		offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer},
		pallet_prelude::*,
	};

	const BATCHING_ENDPOINT_FALLBACK: [u8; 31] = *b"http://0.0.0.0:8070/currencies/";

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The overarching dispatch call type.
		type RuntimeCall: From<Call<Self>>;

		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

		/// Weight of pallet
		type WeightInfo: weights::WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// List of all authorized accounts
	#[pallet::storage]
	#[pallet::getter(fn authorized_accounts)]
	pub type AuthorizedAccounts<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ()>;

	/// List of all supported currencies
	#[pallet::storage]
	#[pallet::getter(fn supported_currencies)]
	pub type SupportedCurrencies<T: Config> = StorageMap<_, Blake2_128Concat, AssetId, ()>;

	#[pallet::storage]
	#[pallet::getter(fn batching_api)]
	pub type BatchingApi<T: Config> = StorageValue<_, Vec<u8>>;

	/// Map of all the coins names to their respective info and price
	#[pallet::storage]
	#[pallet::getter(fn prices_map)]
	pub type CoinInfosMap<T> = StorageMap<_, Blake2_128Concat, AssetId, CoinInfo, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event is triggered when prices are updated
		UpdatedPrices(Vec<((Vec<u8>, Vec<u8>), CoinInfo)>),
		/// Event is triggered when account is authorized
		AccountIdAuthorized(T::AccountId),
		/// Event is triggered when account is deauthorized
		AccountIdDeauthorized(T::AccountId),
		/// Event is triggered when currency is added to the list
		CurrencyAdded(Vec<u8>, Vec<u8>),
		/// Event is triggered when currency is remove from the list
		CurrencyRemoved(Vec<u8>, Vec<u8>),
		/// Event is triggered when batching api route is set from the list
		BatchingApiRouteSet(Vec<u8>),
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
		DeserializeStrError,

		/// Failed Deserializing
		DeserializeError,

		/// Sending Http request to Batching Server Failed
		HttpRequestSendFailed,

		/// Http request to Batching Server Failed
		HttpRequestFailed,

		/// Failed to send signed Transaction
		FailedSignedTransaction,

		/// User cannot deauthorized themself
		UserUnableToDeauthorizeThemself,

		/// BadOrigin
		BadOrigin,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub authorized_accounts: Vec<T::AccountId>,
		pub supported_currencies: Vec<AssetId>,
		pub batching_api: Vec<u8>,
		pub coin_infos_map: Vec<(Vec<u8>, CoinInfo)>,
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for asset_id in &self.supported_currencies {
				<SupportedCurrencies<T>>::insert(asset_id.clone(), ());
			}

			for account_id in &self.authorized_accounts {
				<AuthorizedAccounts<T>>::insert(account_id.clone(), ());
			}
			<BatchingApi<T>>::put(self.batching_api.clone());
		}
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				authorized_accounts: Default::default(),
				supported_currencies: Default::default(),
				batching_api: Default::default(),
				coin_infos_map: Default::default(),
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(_n: T::BlockNumber) {
			match Self::update_prices() {
				Ok(_) => log::info!("Updated Prices"),
				Err(e) => log::error!("Failed to Update Prices {:?}", e),
			}
		}
	}

	impl<T: Config> DiaOracle for Pallet<T> {
		fn get_coin_info(blockchain: Vec<u8>, symbol: Vec<u8>) -> Result<CoinInfo, DispatchError> {
			let asset_id = AssetId { blockchain, symbol };
			ensure!(<CoinInfosMap<T>>::contains_key(&asset_id), Error::<T>::NoCoinInfoAvailable);
			let result = <CoinInfosMap<T>>::get(&asset_id);
			Ok(result)
		}

		fn get_value(blockchain: Vec<u8>, symbol: Vec<u8>) -> Result<PriceInfo, DispatchError> {
			<Pallet<T> as DiaOracle>::get_coin_info(blockchain, symbol)
				.map(|info| PriceInfo { value: info.price })
		}
	}

	impl<T: Config> Pallet<T> {
		fn update_prices() -> Result<(), Error<T>> {
			// Expected contract for the API with the server is supported currencies in URL path and
			// json encoded Vec<CoinInfo> as a result from the server
			let supported_currencies = <SupportedCurrencies<T>>::iter_keys()
				.map(|AssetId { blockchain, symbol }| {
					[
						&b"{\"blockchain\":\""[..],
						&blockchain[..],
						&b"\",\"symbol\":\""[..],
						&symbol[..],
						&b"\"}"[..],
					]
					.concat()
				})
				.collect::<Vec<_>>()
				.join(&b',');

			if supported_currencies.len() == 0 {
				return Ok(());
			}

			let supported_currencies: Vec<_> =
				[&b"{"[..], &supported_currencies[..], &b"}"[..]].concat();

			let api = Self::batching_api()
				.ok_or(<Error<T>>::NoBatchingApiEndPoint) // Error Redundant but Explains Error Reason
				.unwrap_or(BATCHING_ENDPOINT_FALLBACK.to_vec());

			let api = sp_std::str::from_utf8(&api).map_err(|_| <Error<T>>::DeserializeStrError)?;
			let request = offchain::http::Request::post(api, vec![supported_currencies]);

			let pending = request.send().map_err(|_| <Error<T>>::HttpRequestSendFailed)?;
			let response = pending.wait().map_err(|_| <Error<T>>::HttpRequestFailed)?;
			let body = response.body().collect::<Vec<u8>>();

			let prices: Vec<CoinInfo> =
				serde_json::from_slice(&body).map_err(|_| <Error<T>>::DeserializeError)?;

			let prices: Vec<((Vec<u8>, Vec<u8>), CoinInfo)> = prices
				.into_iter()
				.map(|p| ((p.blockchain.clone(), p.symbol.clone()), p))
				.collect();

			let signer = Signer::<T, T::AuthorityId>::any_account();

			log::error!("Signers, {:?}", signer.can_sign());

			signer
				.send_signed_transaction(|account| {
					log::error!("Account, {:?}, {:?}", account.id, account.public);
					Call::<T>::set_updated_coin_infos {
						// `prices` are not `move`d because of Fn(_)
						// `prices` would have `move`d if FnOnce(_) was in signature
						// Hence the redundant clone.
						coin_infos: prices.clone(),
					}
				})
				.ok_or(<Error<T>>::FailedSignedTransaction)?
				.1
				.map_err(|_| <Error<T>>::FailedSignedTransaction)?;

			Ok(())
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
		#[pallet::weight(<T as Config>::WeightInfo::add_currency())]
		pub fn add_currency(
			origin: OriginFor<T>,
			blockchain: Vec<u8>,
			symbol: Vec<u8>,
		) -> DispatchResult {
			let origin_account_id = ensure_signed(origin)?;
			Pallet::<T>::check_origin_rights(&origin_account_id)?;

			let asset_id = AssetId { blockchain: blockchain.clone(), symbol: symbol.clone() };
			if !<SupportedCurrencies<T>>::contains_key(&asset_id) {
				Self::deposit_event(Event::<T>::CurrencyAdded(blockchain, symbol));
				<SupportedCurrencies<T>>::insert(asset_id, ());
			}

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::remove_currency())]
		pub fn remove_currency(
			origin: OriginFor<T>,
			blockchain: Vec<u8>,
			symbol: Vec<u8>,
		) -> DispatchResult {
			let origin_account_id = ensure_signed(origin)?;
			Pallet::<T>::check_origin_rights(&origin_account_id)?;

			let asset_id = AssetId { blockchain: blockchain.clone(), symbol: symbol.clone() };
			if <SupportedCurrencies<T>>::contains_key(&asset_id) {
				Self::deposit_event(Event::<T>::CurrencyRemoved(blockchain, symbol));
				<SupportedCurrencies<T>>::remove(asset_id);
			}

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::authorize_account())]
		pub fn authorize_account(origin: OriginFor<T>, account_id: T::AccountId) -> DispatchResult {
			if let Ok(origin_account_id) = ensure_signed(origin.clone()) {
				Pallet::<T>::check_origin_rights(&origin_account_id)?;
			} else {
				ensure_root(origin)?;
			}

			if !<AuthorizedAccounts<T>>::contains_key(&account_id) {
				Self::deposit_event(Event::<T>::AccountIdAuthorized(account_id.clone()));
				<AuthorizedAccounts<T>>::insert(account_id, ());
			}

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::deauthorize_account())]
		pub fn deauthorize_account(
			origin: OriginFor<T>,
			account_id: T::AccountId,
		) -> DispatchResult {
			if let Ok(origin_account_id) = ensure_signed(origin.clone()) {
				Pallet::<T>::check_origin_rights(&origin_account_id)?;
				ensure!(
					account_id != origin_account_id,
					Error::<T>::UserUnableToDeauthorizeThemself
				);
			} else {
				ensure_root(origin)?;
			}

			if <AuthorizedAccounts<T>>::contains_key(&account_id) {
				Self::deposit_event(Event::<T>::AccountIdDeauthorized(account_id.clone()));
				<AuthorizedAccounts<T>>::remove(account_id);
			}

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::set_updated_coin_infos())]
		pub fn set_updated_coin_infos(
			origin: OriginFor<T>,
			coin_infos: Vec<((Vec<u8>, Vec<u8>), CoinInfo)>,
		) -> DispatchResult {
			let origin_account_id = ensure_signed(origin)?;
			Pallet::<T>::check_origin_rights(&origin_account_id)?;
			Self::deposit_event(Event::<T>::UpdatedPrices(coin_infos.clone()));
			for ((blockchain, symbol), c) in coin_infos {
				<CoinInfosMap<T>>::insert(AssetId { blockchain, symbol }, c);
			}
			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::set_batching_api())]
		pub fn set_batching_api(origin: OriginFor<T>, api: Vec<u8>) -> DispatchResult {
			let origin_account_id = ensure_signed(origin)?;
			Pallet::<T>::check_origin_rights(&origin_account_id)?;
			<BatchingApi<T>>::put(api.clone());
			Self::deposit_event(Event::<T>::BatchingApiRouteSet(api));
			Ok(())
		}
	}
}
