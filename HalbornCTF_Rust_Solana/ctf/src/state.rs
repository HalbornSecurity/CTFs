use borsh::{
    BorshDeserialize,
    BorshSerialize
};
use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey
};

/// Account holding game configuration
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GameConfig {
    account_type: AccountType,

    /// credits per level
    pub credits_per_level: u8
}

impl GameConfig {
    /// Create a new game configuration account
    pub fn new(
        credits_per_level: u8
    ) -> Self {
        Self {
            account_type: AccountType::GameConfig,
            credits_per_level
        }
    }
}

/// Account holding user data
#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct User {
    account_type: AccountType,

    /// address of the account owner
    pub authority: Pubkey,

    /// address of the game config this user user selected
    pub game_config: Pubkey,
    
    /// current credits
    /// burn to increase level
    pub credits: u32,
    
    /// user level
    pub level: u8
}

impl User {
    /// Create a new User account
    pub fn new(
        authority: &Pubkey,
        game_config: &Pubkey
    ) -> Self {
        Self {
            account_type: AccountType::User,
            authority: *authority,
            game_config: *game_config,
            ..User::default()
        }
    }
}

/// Account types defined in this program
#[derive(PartialEq, Debug, BorshSerialize, BorshDeserialize)]
pub enum AccountType {
    Uninitialized = 0,
    GameConfig,
    User
}

impl Default for AccountType {
    /// Accounts are uinitialized by default
    fn default() -> Self { AccountType::Uninitialized }
}

pub trait AccountData {
    /// Return the type of the provided account or throw an error if type lookup failed
    fn try_get_type(&self) -> Result<AccountType, ProgramError>;
}

impl AccountData for AccountInfo<'_> {
    fn try_get_type(&self) -> Result<AccountType, ProgramError> {
        if !crate::check_id(self.owner) {
            return Err(ProgramError::IllegalOwner)
        }

        match self.try_borrow_data()?.get(0).ok_or(ProgramError::AccountBorrowFailed)? {
            1 => Ok(AccountType::GameConfig),
            2 => Ok(AccountType::User),
            _ => Err(ProgramError::InvalidAccountData)
        }
    }
}