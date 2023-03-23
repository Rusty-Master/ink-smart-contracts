#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::let_unit_value)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {
    use ink_storage::{traits::SpreadAllocate, Mapping};

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientAllowance,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(event)]
    pub struct Transfer {
        // #[ink(topic)] - annotated fields are indexed
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Erc20 {
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
        //(owner, spender) -> allowed
        allowances: Mapping<(AccountId, AccountId), Balance>,
    }

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            ink_lang::utils::initialize_contract(|contract| {
                Self::new_init(contract, initial_supply);
            })
        }

        fn new_init(&mut self, initial_supply: Balance) {
            let caller = Self::env().caller();
            self.balances.insert(caller, &initial_supply);
            self.total_supply = initial_supply;
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
            })
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)
        }

        fn transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of_impl(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(from, &(from_balance - value));
            let to_balance = self.balance_of_impl(to);
            self.balances.insert(to, &(to_balance + value));
            Self::env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
            });
            Ok(())
        }

        #[inline]
        fn balance_of_impl(&self, owner: &AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((&owner, &spender), &value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_impl(&owner, &spender)
        }

        #[inline]
        fn allowance_impl(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }

        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance_impl(&from, &caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance);
            }
            self.transfer_from_to(&from, &to, value)?;
            self.allowances
                .insert((&from, &caller), &(allowance - value));
            Ok(())
        }
    }

    // TODO: you might add tests that exercise error handling for invalid input, empty values, or values that are out of expected bounds.

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        #[ink::test]
        fn new_works() {
            let contract = Erc20::new(777);
            assert_eq!(contract.total_supply(), 777);
        }

        #[ink::test]
        fn balance_works() {
            let contract = Erc20::new(777);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 777);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 0);
        }

        #[ink::test]
        fn transfer_works() {
            let mut erc20 = Erc20::new(100);
            assert_eq!(erc20.balance_of(AccountId::from([0x0; 32])), 0);
            assert_eq!(erc20.transfer(AccountId::from([0x0; 32]), 10), Ok(()));
            assert_eq!(erc20.balance_of(AccountId::from([0x0; 32])), 10);
            assert_eq!(2, ink_env::test::recorded_events().count());
        }

        #[ink::test]
        fn transfer_from_works() {
            let mut erc20 = Erc20::new(100);
            assert_eq!(erc20.balance_of(AccountId::from([0x1; 32])), 100);
            erc20.approve(AccountId::from([0x1; 32]), 20);
            erc20.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 10);
            assert_eq!(erc20.balance_of(AccountId::from([0x0; 32])), 10);
        }

        #[ink::test]
        fn allowances_works() {
            let mut erc20 = Erc20::new(100);
            assert_eq!(erc20.balance_of(AccountId::from([0x1; 32])), 100);
            erc20.approve(AccountId::from([0x1; 32]), 200);
            assert_eq!(
                erc20.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])),
                200
            );
            erc20.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 50);
            assert_eq!(erc20.balance_of(AccountId::from([0x0; 32])), 50);
            assert_eq!(
                erc20.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])),
                150
            );

            erc20.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 100);
            assert_eq!(erc20.balance_of(AccountId::from([0x0; 32])), 50);
            assert_eq!(
                erc20.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])),
                150
            );
        }
    }
}
