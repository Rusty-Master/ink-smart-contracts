use openbrush::{
    contracts::{
        reentrancy_guard::ReentrancyGuardError,
        traits::{
            ownable::*,
            psp22::PSP22Error,
        },
    },
    traits::{
        AccountId,
        Balance,
        Timestamp,
    },
};

#[openbrush::wrapper]
pub type PairRef = dyn Pair;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PairError {
    PSP22Error(PSP22Error),
    ReentrancyGuardError(ReentrancyGuardError),
    OwnableError(OwnableError),
    InsufficientLiquidityMinted,
    Overflow,
    SubUnderFlow1,
    SubUnderFlow2,
    SubUnderFlow3,
    SubUnderFlow14,
    MulOverFlow1,
    MulOverFlow2,
    MulOverFlow3,
    MulOverFlow4,
    MulOverFlow5,
    MulOverFlow14,
    MulOverFlow15,
    DivByZero1,
    DivByZero2,
    DivByZero5,
    AddOverflow1,
    MulOverFlow6,
    DivByZero3,
    MulOverFlow7,
    DivByZero4,
    InsufficientLiquidityBurned,
    MulOverFlow8,
    InsufficientInputAmount,
    SubUnderFlow10,
    MulOverFlow10,
    SubUnderFlow11,
    MulOverFlow16,
    MulOverFlow17,
    MulOverFlow18,
    K,
    SubUnderFlow9,
    InsufficientOutputAmout,
    InsufficientLiquidity,
    InvalidTo,
    SubUnderFlow4,
    SubUnderFlow6,
    SubUnderFlow5,
    SubUnderFlow7,
    SubUnderFlow8,
    MulOverFlow9,
    MulOverFlow11,
}

impl From<PSP22Error> for PairError {
    fn from(error: PSP22Error) -> Self {
        PairError::PSP22Error(error)
    }
}

impl From<ReentrancyGuardError> for PairError {
    fn from(error: ReentrancyGuardError) -> Self {
        PairError::ReentrancyGuardError(error)
    }
}

impl From<OwnableError> for PairError {
    fn from(error: OwnableError) -> Self {
        PairError::OwnableError(error)
    }
}

#[openbrush::trait_definition]
pub trait Pair {
    #[ink(message)]
    fn get_reserves(&self) -> (Balance, Balance, Timestamp);

    #[ink(message)]
    fn initialize(&mut self, token_0: AccountId, token_1: AccountId) -> Result<(), PairError>;

    #[ink(message)]
    fn get_token_0(&self) -> AccountId;

    #[ink(message)]
    fn get_token_1(&self) -> AccountId;

    #[ink(message)]
    fn mint(&mut self, to: AccountId) -> Result<Balance, PairError>;

    #[ink(message)]
    fn burn(&mut self, to: AccountId) -> Result<(Balance, Balance), PairError>;

    #[ink(message)]
    fn swap(
        &mut self,
        amount_0_out: Balance,
        amount_1_out: Balance,
        to: AccountId,
    ) -> Result<(), PairError>;

    fn _safe_transfer(
        &mut self,
        token: AccountId,
        to: AccountId,
        value: Balance,
    ) -> Result<(), PairError>;

    fn _mint_fee(&mut self, reserve_0: Balance, reserve_1: Balance) -> Result<bool, PairError>;

    fn _update(
        &mut self,
        balance_0: Balance,
        balance_1: Balance,
        reserve_0: Balance,
        reserve_1: Balance,
    ) -> Result<(), PairError>;

    fn _emit_mint_event(&self, _sender: AccountId, _amount_0: Balance, _amount_1: Balance);

    fn _emit_sync_event(&self, reserve_0: Balance, reserve_1: Balance);

    fn _emit_burn_event(
        &self,
        _sender: AccountId,
        _amount_0: Balance,
        _amount_1: Balance,
        _to: AccountId,
    );

    fn _emit_swap_event(
        &self,
        _sender: AccountId,
        _amount_0_in: Balance,
        _amount_1_in: Balance,
        _amount_0_out: Balance,
        _amount_1_out: Balance,
        _to: AccountId,
    );
}
