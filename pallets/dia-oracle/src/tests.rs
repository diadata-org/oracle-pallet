use crate::mock::*;
use crate::*;

use frame_support::assert_err;
use sp_core::sr25519::Public;

pub const ALICE: Public = Public([1u8; 32]);

#[test]
fn add_currency_should_work() {
	new_test_ext().execute_with(|| {
		<AuthorizedAccounts<Test>>::insert(ALICE, ());

		let _test1 = DOracle::add_currency(RuntimeOrigin::signed(ALICE), vec![1]);
		let _test2 = DOracle::add_currency(RuntimeOrigin::signed(ALICE), vec![2]);
		let _test3 = DOracle::add_currency(RuntimeOrigin::signed(ALICE), vec![3]);

		assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![1]), true);
		assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![2]), true);
		assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![3]), true);
		assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![4]), false);
	})
}

#[test]
fn remove_currency_should_work() {
	new_test_ext().execute_with(|| {
		<AuthorizedAccounts<Test>>::insert(ALICE, ());

		let _test1 = DOracle::add_currency(RuntimeOrigin::signed(ALICE), vec![1]);
		let _test2 = DOracle::add_currency(RuntimeOrigin::signed(ALICE), vec![2]);
		let _test3 = DOracle::remove_currency(RuntimeOrigin::signed(ALICE), vec![2]);

		assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![1]), true);
		assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![2]), false);
	})
}

#[test]
fn authorize_account_should_work() {
	new_test_ext().execute_with(|| {
		<AuthorizedAccounts<Test>>::insert(get_account_id(1), ());

		let _test1 =
			DOracle::authorize_account(RuntimeOrigin::signed(get_account_id(1)), get_account_id(2));
		let _test2 =
			DOracle::authorize_account(RuntimeOrigin::signed(get_account_id(1)), get_account_id(3));
		let _test3 =
			DOracle::authorize_account(RuntimeOrigin::signed(get_account_id(1)), get_account_id(4));
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
			DOracle::authorize_account(RuntimeOrigin::signed(get_account_id(1)), get_account_id(1));
		let _test2 =
			DOracle::authorize_account(RuntimeOrigin::signed(get_account_id(2)), get_account_id(2));
		let _test3 =
			DOracle::authorize_account(RuntimeOrigin::signed(get_account_id(3)), get_account_id(3));

		let _test4 = DOracle::deauthorize_account(
			RuntimeOrigin::signed(get_account_id(3)),
			get_account_id(1),
		);
		let _test5 = DOracle::deauthorize_account(
			RuntimeOrigin::signed(get_account_id(3)),
			get_account_id(2),
		);

		assert_eq!(<AuthorizedAccounts<Test>>::contains_key(get_account_id(1)), false);
		assert_eq!(<AuthorizedAccounts<Test>>::contains_key(get_account_id(2)), false);
		assert_eq!(<AuthorizedAccounts<Test>>::contains_key(get_account_id(3)), true);
	})
}

#[test]
fn deauthorize_account_should_not_work_by_deauthorizing_themself() {
	new_test_ext().execute_with(|| {
		<AuthorizedAccounts<Test>>::insert(get_account_id(1), ());
		<AuthorizedAccounts<Test>>::insert(get_account_id(2), ());

		let _test1 =
			DOracle::authorize_account(RuntimeOrigin::signed(get_account_id(1)), get_account_id(1));
		let _test2 =
			DOracle::authorize_account(RuntimeOrigin::signed(get_account_id(1)), get_account_id(2));
		let _test3 = DOracle::deauthorize_account(
			RuntimeOrigin::signed(get_account_id(2)),
			get_account_id(2),
		);
		let _test4 = DOracle::deauthorize_account(
			RuntimeOrigin::signed(get_account_id(2)),
			get_account_id(1),
		);

		assert_eq!(<AuthorizedAccounts<Test>>::contains_key(get_account_id(1)), false);
		assert_eq!(<AuthorizedAccounts<Test>>::contains_key(get_account_id(2)), true);
	})
}

#[test]
fn set_updated_coin_infos_should_work() {
	new_test_ext().execute_with(|| {
		<AuthorizedAccounts<Test>>::insert(ALICE, ());

		let example_info: CoinInfo = CoinInfo {
			symbol: vec![1],
			name: vec![1],
			supply: 9,
			last_update_timestamp: 9,
			price: 9,
		};
		let coin_infos =
			vec![(vec![1, 2, 3], CoinInfo::default()), (vec![2, 2, 2], example_info.clone())];
		let _test1 = DOracle::set_updated_coin_infos(RuntimeOrigin::signed(ALICE), coin_infos);

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

		let _test1 = DOracle::add_currency(RuntimeOrigin::signed(get_account_id(1)), vec![1]);
		let _test2 = DOracle::add_currency(RuntimeOrigin::signed(get_account_id(2)), vec![2]);

		assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![1]), true);
		assert_eq!(<SupportedCurrencies<Test>>::contains_key(vec![2]), true);
	})
}

#[test]
fn get_coin_info_should_work() {
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
			RuntimeOrigin::signed(get_account_id(1)),
			coin_infos.clone(),
		);

		let coin_info = DOracle::get_coin_info(vec![2, 2, 2]);

		assert_eq!(coin_info, Ok(example_info));
		assert_eq!(Ok(9), DOracle::get_value(vec![2, 2, 2]).map(|x| x.value));
	})
}

#[test]
fn get_coin_info_should_return_error() {
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
			RuntimeOrigin::signed(get_account_id(1)),
			coin_infos.clone(),
		);

		let fail_coin_info = DOracle::get_coin_info(vec![1, 2, 3, 4]);

		assert_err!(fail_coin_info, Error::<Test>::NoCoinInfoAvailable);
	})
}

#[test]
fn get_value_in_coin_info_should_work() {
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
			RuntimeOrigin::signed(get_account_id(1)),
			coin_infos.clone(),
		);

		let coin_info_one = DOracle::get_value(vec![1, 2, 3]);
		let coin_info_two = DOracle::get_value(vec![2, 2, 2]);

		assert_eq!(coin_info_one.map(|x| x.value), Ok(CoinInfo::default().price));
		assert_eq!(coin_info_two.map(|x| x.value), Ok(example_info.price));
	})
}

#[test]
fn get_value_in_coin_info_should_return_error() {
	new_test_ext().execute_with(|| {
		<AuthorizedAccounts<Test>>::insert(get_account_id(1), ());

		let coin_infos = vec![(vec![1, 2, 3], CoinInfo::default())];

		let _test1 = DOracle::set_updated_coin_infos(
			RuntimeOrigin::signed(get_account_id(1)),
			coin_infos.clone(),
		);

		let fail_coin_info = DOracle::get_value(vec![1, 2, 3, 4]);

		assert_err!(fail_coin_info, Error::<Test>::NoCoinInfoAvailable);
	})
}
