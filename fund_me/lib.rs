#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::let_unit_value)]
#![allow(clippy::new_without_default)]

use ink_lang as ink;
#[ink::contract]
mod fund_me {

    use diadata::DiadataRef;
    use ink_prelude::vec::Vec;
    use ink_storage::{traits::SpreadAllocate, Mapping};

    #[derive(Debug)]
    pub enum Error {
        MulOverFlow,
        DivByZero,
    }

    const MIN_PAYABLE_VALUE: Balance = 50;
    #[derive(SpreadAllocate)]
    #[ink(storage)]
    pub struct FundMe {
        diadata: AccountId,
        address_to_amount: Mapping<AccountId, Balance>,
        founders: Vec<AccountId>,
        owner: AccountId,
    }

    impl FundMe {
        #[ink(constructor)]
        pub fn new(oracle_address: AccountId) -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.founders = Vec::new();
                contract.diadata = oracle_address;
                contract.owner = Self::env().caller();
            })
        }

        #[ink(message)]
        pub fn withdraw(&mut self) {
            ink_env::debug_println!("Request to withdraw funds");

            assert_eq!(
                self.env().caller(),
                self.owner,
                "Insufficent permissions. Caller {:?} does not have sufficient permissions, only {:?} does",
                self.env().caller(),
                self.owner,
            );

            // TODO: Existential deposit should be left ????
            ink_env::debug_println!("Amount: {}", self.env().balance());
            if self
                .env()
                .transfer(self.env().caller(), self.env().balance())
                .is_err()
            {
                panic!("You should leave existential deposit probably")
            };
        }

        #[ink(message, payable)]
        pub fn fund(&mut self) {
            let endowment = self.env().transferred_value();
            let caller = self.env().caller();
            let caller_balance = self.address_to_amount.get(caller).unwrap_or(0);
            ink_env::debug_println!("Contract balance: {}", self.env().balance());
            // check if payment > 50 bucks or founder already payed minimum
            let is_founder = self.founders.contains(&caller);
            assert!(is_founder || is_min_payment(self.diadata, endowment));

            // save founder in storage
            self.address_to_amount
                .insert(caller, &(caller_balance + endowment));
            if !is_founder {
                self.founders.push(self.env().caller());
            }
        }

        //TODO: shoud cover situation when funds send directly not by fund method

        #[ink(message)]
        pub fn get_balance(&self) -> Balance {
            self.env().balance()
        }

        #[ink(message)]
        pub fn am_i_founder(&self) -> bool {
            self.founders.contains(&self.env().caller())
        }

        #[ink(message)]
        pub fn set_oracle_address(&mut self, oracle_address: AccountId) {
            self.diadata = oracle_address;
        }
    }

    fn is_min_payment(oracle_address: AccountId, value: Balance) -> bool {
        let oracle: DiadataRef = ink_env::call::FromAccountId::from_account_id(oracle_address);
        let price = oracle
            .get("SDN/USD".into())
            .price
            .checked_mul(10000)
            .ok_or(Error::MulOverFlow)
            .unwrap();
        ink_env::debug_println!("SDN/USD: {:?}", price);
        let payment = value
            .checked_mul(price as u128)
            .unwrap()
            .checked_div(1000000000000)
            .unwrap();

        let min_payment = MIN_PAYABLE_VALUE.checked_mul(1000000000000).unwrap();

        ink_env::debug_println!("payment: {}", payment);
        ink_env::debug_println!("min_payment: {}", min_payment);

        payment > min_payment
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        //TODO: Lets try to use dia oracle to set a limit to a minimum of 50 $ worth of tokens to be a founder
        #[ink::test]
        fn founding_works() {
            // this allows me to call fund_me.env()
            use ink::codegen::Env;

            // given
            let mut fund_me = FundMe::new(AccountId::from([0x1; 32]));

            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            let contract_id = fund_me.env().account_id();
            // TODO: export ink_env calls to helper functions
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(accounts.eve, 100);
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(contract_id, 0);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.eve);

            // when
            ink_env::pay_with_call!(fund_me.fund(), 10);

            //then
            assert_eq!(
                ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.eve)
                    .expect("Cannot get account balance"),
                90
            );
            assert_eq!(fund_me.get_balance(), 10);
        }

        #[ink::test]
        fn withdraw_works() {
            use ink::codegen::Env;
            // given
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.eve);

            let mut fund_me = FundMe::new(AccountId::from([0x1; 32]));
            let contract_id = fund_me.env().account_id();
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(contract_id, 100);

            // when
            fund_me.withdraw();

            // then
            assert_eq!(
                ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.eve)
                    .expect("Cannot get account balance"),
                100
            );
            assert_eq!(fund_me.get_balance(), 0);
        }

        #[ink::test]
        #[should_panic(expected = "Insufficent permissions.")]
        fn withdraw_panics() {
            use ink::codegen::Env;
            // given
            // contract created by default account
            let mut fund_me = FundMe::new(AccountId::from([0x1; 32]));

            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            let contract_id = fund_me.env().account_id();

            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(contract_id, 100);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.eve);

            // when
            // withd
            fund_me.withdraw();

            // then
            assert_eq!(
                ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.eve)
                    .expect("Cannot get account balance"),
                100
            );
            assert_eq!(fund_me.get_balance(), 0);
        }
    }
}
