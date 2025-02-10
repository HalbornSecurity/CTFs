use borsh::{BorshSerialize, BorshDeserialize};
use solana_program::{
    account_info::{
        next_account_info,
        AccountInfo
    },
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account, 
};
use crate::{
    state::*,
    constants::*
};

/// Create a new game configuration account and set credits per level
pub fn create_game_config(
    credits_per_level: u8,
    accounts: &[AccountInfo]
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let game_config_info = next_account_info(accounts_iter)?;
    let admin_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    let (game_config_pubkey, bump) = Pubkey::find_program_address(&[
        admin_info.signer_key().ok_or(ProgramError::MissingRequiredSignature)?.as_ref(),
        GAME_CONFIG_SEED
    ],
    &crate::id()
    );

    assert_eq!(game_config_info.key, &game_config_pubkey);

    if !game_config_info.try_data_is_empty()? {
        return Err(ProgramError::AccountAlreadyInitialized)
    }

    invoke_signed(
        &create_account(
            admin_info.signer_key().ok_or(ProgramError::MissingRequiredSignature)?,
            game_config_info.key,
            Rent::default().minimum_balance(std::mem::size_of::<GameConfig>()),
            std::mem::size_of::<GameConfig>().try_into().map_err(|_| ProgramError::MaxAccountsDataAllocationsExceeded)?,
            &crate::id()
        ),
        &[
            admin_info.clone(),
            game_config_info.clone(),
            system_program_info.clone()
        ],
        &[&[admin_info.key.as_ref(), GAME_CONFIG_SEED, &[bump]]]
    )?;

    let game_config = GameConfig::new(
        credits_per_level
    );

    game_config.serialize(&mut game_config_info.try_borrow_mut_data()?.as_mut())?;

    Ok(())
}

/// Create a new user account, select game configuration and set user authority
pub fn create_user(
    accounts: &[AccountInfo]
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let game_config_info = next_account_info(accounts_iter)?;
    let user_info = next_account_info(accounts_iter)?;
    let authority_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    assert_eq!(game_config_info.try_get_type()?, AccountType::GameConfig);
    
    let (user_pubkey, bump) = Pubkey::find_program_address(&[
            game_config_info.key.as_ref(),
            authority_info.signer_key().ok_or(ProgramError::MissingRequiredSignature)?.as_ref(),
            USER_SEED
        ],
        &crate::id()
    );

    assert_eq!(user_info.key, &user_pubkey);

    invoke_signed(
        &create_account(
            authority_info.key,
            user_info.key,
            Rent::default().minimum_balance(std::mem::size_of::<User>()),
            std::mem::size_of::<User>().try_into().map_err(|_| ProgramError::MaxAccountsDataAllocationsExceeded)?,
            &crate::id()
        ),
        &[
            authority_info.clone(),
            user_info.clone(),
            system_program_info.clone()
        ],
        &[&[game_config_info.key.as_ref(), authority_info.key.as_ref(), USER_SEED, &[bump]]]
    )?;

    let user = User::new(
        authority_info.key,
        game_config_info.key
    );

    user.serialize(&mut user_info.try_borrow_mut_data()?.as_mut())?;

    Ok(())
}

/// Mint credits to a user account as game configuration admin
pub fn mint_credits_to_user(
    credits: u32,
    accounts: &[AccountInfo]
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let game_config_info = next_account_info(accounts_iter)?;
    let user_info = next_account_info(accounts_iter)?;
    let admin_info = next_account_info(accounts_iter)?;

    assert_eq!(game_config_info.try_get_type()?, AccountType::GameConfig);
    assert_eq!(user_info.try_get_type()?, AccountType::User);

    let (game_config_pubkey, _) = Pubkey::find_program_address(&[
        admin_info.signer_key().ok_or(ProgramError::MissingRequiredSignature)?.as_ref(),
        GAME_CONFIG_SEED
    ],
    &crate::id()
    );

    assert_eq!(game_config_info.key, &game_config_pubkey);

    let mut user = User::deserialize(&mut user_info.try_borrow_mut_data()?.as_ref())?;

    assert_eq!(&user.game_config, game_config_info.key);

    user.credits += credits;
    user.serialize(&mut user_info.try_borrow_mut_data()?.as_mut())?;

    Ok(())
}

/// Level up as user
pub fn user_level_up(
    credits_to_burn: u32,
    accounts: &[AccountInfo]
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let game_config_info = next_account_info(accounts_iter)?;
    let user_info = next_account_info(accounts_iter)?;
    let authority_info = next_account_info(accounts_iter)?;

    assert_eq!(game_config_info.try_get_type()?, AccountType::GameConfig);
    assert_eq!(user_info.try_get_type()?, AccountType::User);

    let game_config = GameConfig::deserialize(&mut game_config_info.try_borrow_data()?.as_ref())?;
    let mut user = User::deserialize(&mut user_info.try_borrow_data()?.as_ref())?;

    assert_eq!(authority_info.signer_key().ok_or(ProgramError::MissingRequiredSignature)?, &user.authority);

    let mut iterator: u8 = user.level; 
    let mut level_credits = iterator as u32 * game_config.credits_per_level as u32;
    let mut next_level_credits = level_credits;
    let mut stop = false;
    
    while next_level_credits < credits_to_burn && !stop {
        level_credits = next_level_credits;
        
        if iterator < MAX_LEVEL {
            iterator += 1; 
            next_level_credits += iterator as u32 * game_config.credits_per_level as u32;
        } else {
            stop = true;
        }

    }

    user.credits -= level_credits;
    
    if !(user.credits > 0) {
        return Err(ProgramError::InsufficientFunds)
    }

    user.level = iterator;

    user.serialize(&mut user_info.try_borrow_mut_data()?.as_mut())?;

    Ok(())
}