#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]
pub use pallet::*;

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
		ensure_signed,
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

		/// User cannot deauthorized themself
		UserUnableToDeauthorizeThemself,
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
		fn get_coin_info(name: Vec<u8>) -> Result<CoinInfo, DispatchError> {
			ensure!(<CoinInfosMap<T>>::contains_key(&name), Error::<T>::NoCoinInfoAvailable);
			let result = <CoinInfosMap<T>>::get(name);
			Ok(result)
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
	use frame_support::{assert_err, parameter_types};
	use frame_system as system;
	use sp_core::{sr25519::Signature, H256};
	use sp_runtime::{
		testing::{Header, TestXt},
		traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
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
		type AccountId = sp_core::sr25519::Public;
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
	type Extrinsic = TestXt<Call, ()>;
	type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

	impl frame_system::offchain::SigningTypes for Test {
		type Public = <Signature as Verify>::Signer;
		type Signature = Signature;
	}

	impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
	where
		Call: From<LocalCall>,
	{
		type OverarchingCall = Call;
		type Extrinsic = Extrinsic;
	}

	impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
	where
		Call: From<LocalCall>,
	{
		fn create_transaction<
			C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>,
		>(
			call: Call,
			_public: <Signature as Verify>::Signer,
			_account: AccountId,
			nonce: u64,
		) -> Option<(Call, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
			Some((call, (nonce, ())))
		}
	}

	impl dia_oracle::Config for Test {
		type Event = Event;
		type AuthorityId = super::crypto::DiaAuthId;
		type Call = Call;
	}

	// Build genesis storage according to the mock runtime.
	pub fn new_test_ext() -> sp_io::TestExternalities {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	fn get_account_id(id: u8) -> AccountId {
		AccountId::from(sp_core::sr25519::Public::from_raw([id; 32]))
	}
	#[test]
	fn add_currency_should_work() {
		new_test_ext().execute_with(|| {
			<AuthorizedAccounts<Test>>::insert(AccountId::default(), ());

			let _test1 = DOracle::add_currency(Origin::signed(Default::default()), vec![1]);
			let _test2 = DOracle::add_currency(Origin::signed(Default::default()), vec![2]);
			let _test3 = DOracle::add_currency(Origin::signed(Default::default()), vec![3]);

			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![1]), true);
			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![2]), true);
			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![3]), true);
			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![4]), false);
		})
	}

	#[test]
	fn remove_currency_should_work() {
		new_test_ext().execute_with(|| {
			<AuthorizedAccounts<Test>>::insert(AccountId::default(), ());

			let _test1 = DOracle::add_currency(Origin::signed(Default::default()), vec![1]);
			let _test2 = DOracle::add_currency(Origin::signed(Default::default()), vec![2]);
			let _test3 = DOracle::remove_currency(Origin::signed(Default::default()), vec![2]);

			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![1]), true);
			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![2]), false);
		})
	}

	#[test]
	fn authorize_account_should_work() {
		new_test_ext().execute_with(|| {
			<AuthorizedAccounts<Test>>::insert(get_account_id(1), ());

			let _test1 =
				DOracle::authorize_account(Origin::signed(get_account_id(1)), get_account_id(2));
			let _test2 =
				DOracle::authorize_account(Origin::signed(get_account_id(1)), get_account_id(3));
			let _test3 =
				DOracle::authorize_account(Origin::signed(get_account_id(1)), get_account_id(4));
			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(get_account_id(2)), true);
			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(get_account_id(3)), true);
			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(get_account_id(4)), true);
		})
	}

	#[test]
	fn deauthorize_account_should_work_without_deauthorizing_themself() {
		new_test_ext().execute_with(|| {
			<AuthorizedAccounts<Test>>::insert(get_account_id(1), ());
			<AuthorizedAccounts<Test>>::insert(get_account_id(2), ());
			<AuthorizedAccounts<Test>>::insert(get_account_id(3), ());

			let _test1 =
				DOracle::authorize_account(Origin::signed(get_account_id(1)), get_account_id(1));
			let _test2 =
				DOracle::authorize_account(Origin::signed(get_account_id(2)), get_account_id(2));
			let _test3 =
				DOracle::authorize_account(Origin::signed(get_account_id(3)), get_account_id(3));

			let _test4 =
				DOracle::deauthorize_account(Origin::signed(get_account_id(3)), get_account_id(1));
			let _test5 =
				DOracle::deauthorize_account(Origin::signed(get_account_id(3)), get_account_id(2));

			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(get_account_id(1)), false);
			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(get_account_id(2)), false);
			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(get_account_id(3)), true);
		})
	}

	#[test]
	fn deauthorize_account_should_not_work_ny_deauthorizing_themself() {
		new_test_ext().execute_with(|| {
			<AuthorizedAccounts<Test>>::insert(get_account_id(1), ());
			<AuthorizedAccounts<Test>>::insert(get_account_id(2), ());

			let _test1 =
				DOracle::authorize_account(Origin::signed(get_account_id(1)), get_account_id(1));
			let _test2 =
				DOracle::authorize_account(Origin::signed(get_account_id(1)), get_account_id(2));
			let _test3 =
				DOracle::deauthorize_account(Origin::signed(get_account_id(2)), get_account_id(2));
			let _test4 =
				DOracle::deauthorize_account(Origin::signed(get_account_id(2)), get_account_id(1));

			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(get_account_id(1)), false);
			assert_eq!(<AuthorizedAccounts<Test>>::contains_key(get_account_id(2)), true);
		})
	}

	#[test]
	fn set_updated_coin_infos_should_work() {
		new_test_ext().execute_with(|| {
			<AuthorizedAccounts<Test>>::insert(AccountId::default(), ());

			let example_info: CoinInfo = CoinInfo {
				symbol: vec![1],
				name: vec![1],
				supply: 9,
				last_update_timestamp: 9,
				price: 9,
			};
			let coin_infos =
				vec![(vec![1, 2, 3], CoinInfo::default()), (vec![2, 2, 2], example_info.clone())];
			let _test1 =
				DOracle::set_updated_coin_infos(Origin::signed(Default::default()), coin_infos);

			assert_eq!(<CoinInfosMap<Test>>::contains_key(vec![1, 2, 3]), true);
			assert_eq!(<CoinInfosMap<Test>>::contains_key(vec![2, 2, 2]), true);
			assert_eq!(<CoinInfosMap<Test>>::get(vec![2, 2, 2]), example_info);
			assert_eq!(<CoinInfosMap<Test>>::get(vec![1, 2, 3]), CoinInfo::default());
		})
	}

	#[test]
	fn check_origin_right_shoud_work() {
		new_test_ext().execute_with(|| {
			<AuthorizedAccounts<Test>>::insert(get_account_id(1), ());
			<AuthorizedAccounts<Test>>::insert(get_account_id(2), ());

			let _test1 = DOracle::add_currency(Origin::signed(get_account_id(1)), vec![1]);
			let _test2 = DOracle::add_currency(Origin::signed(get_account_id(2)), vec![2]);

			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![1]), true);
			assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![2]), true);
		})
	}

	#[test]
	fn get_coin_info_shoud_work() {
		new_test_ext().execute_with(|| {
			<AuthorizedAccounts<Test>>::insert(get_account_id(1), ());

			let example_info: CoinInfo = CoinInfo {
				symbol: vec![1],
				name: vec![1],
				supply: 9,
				last_update_timestamp: 9,
				price: 9,
			};
			let coin_infos =
				vec![(vec![1, 2, 3], CoinInfo::default()), (vec![2, 2, 2], example_info.clone())];

			let _test1 = DOracle::set_updated_coin_infos(
				Origin::signed(get_account_id(1)),
				coin_infos.clone(),
			);

			let coin_info = DOracle::get_coin_info(vec![2, 2, 2]);
			let fail_coin_info = DOracle::get_coin_info(vec![1, 2, 3, 4]);

			assert_eq!(coin_info, Ok(example_info));
			assert_eq!(Ok(9), DOracle::get_value(vec![2, 2, 2]));
			assert_err!(fail_coin_info, Error::<Test>::NoCoinInfoAvailable);
		})
	}
}
