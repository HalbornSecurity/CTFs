use borsh::{
    BorshDeserialize,
    BorshSerialize
};
use solana_program::{
    instruction::{
        Instruction,
        AccountMeta
    },
    pubkey::Pubkey,
    system_program
};
use crate::constants::*;

/// Program instructions
/// accounts:
/// [s] signer
/// [w] writable
/// [r] read-only
#[derive(BorshSerialize, BorshDeserialize)]
pub enum ProgramInstruction {
    /// Create a new GameConfig account
    /// accounts:
    /// [ws] game config
    /// [s] admin
    /// [r] system program
    CreateGameConfig {
        credits_per_level: u8
    },

    /// Create a new User account
    /// accounts:
    /// [r] game config
    /// [w] user 
    /// [s] user authority
    /// [r] system program
    CreateUser { },

    /// Mint credits to a User account
    /// accounts:
    /// [r] game config
    /// [w] user account
    /// [s] admin
    MintCreditsToUser {
        credits: u32
    },
    
    /// Level up a User account
    /// accounts:
    /// [r] game config
    /// [w] user account
    /// [s] user authority
    /// [r] system program
    UserLevelUp {
        credits_to_burn: u32
    }
}

/// Create a `CreateGameConfig` instruction
pub fn create_game_config(
    game_config: Pubkey,
    admin: Pubkey,
    credits_per_level: u8
) -> Instruction {
    Instruction::new_with_borsh(
        crate::id(),
        &ProgramInstruction::CreateGameConfig {
            credits_per_level
        },
        vec![
            AccountMeta::new(game_config, NOT_A_SIGNER),
            AccountMeta::new_readonly(admin, SIGNER),
            AccountMeta::new_readonly(system_program::id(), NOT_A_SIGNER)
        ]
    )
}

/// Create a `CreateUser` instruction
pub fn create_user(
    game_config: Pubkey,
    user: Pubkey,
    user_authority: Pubkey,
) -> Instruction {
    Instruction::new_with_borsh(
        crate::id(),
        &ProgramInstruction::CreateUser { },
        vec![
            AccountMeta::new_readonly(game_config, NOT_A_SIGNER),
            AccountMeta::new(user, NOT_A_SIGNER),
            AccountMeta::new(user_authority, SIGNER),
            AccountMeta::new_readonly(system_program::id(), NOT_A_SIGNER)
        ]
    )
}

/// Create a `MintCreditsToUser` instruction
pub fn mint_credits_to_user(
    game_config: Pubkey,
    user_account: Pubkey,
    admin: Pubkey,
    credits: u32
) -> Instruction {
    Instruction::new_with_borsh(
        crate::id(),
        &ProgramInstruction::MintCreditsToUser {
            credits
        },
        vec![
            AccountMeta::new_readonly(game_config, NOT_A_SIGNER),
            AccountMeta::new(user_account, NOT_A_SIGNER),
            AccountMeta::new_readonly(admin, SIGNER)
        ]
    )
}

/// Create a `UserLevelUp` instruction
pub fn user_level_up(
    game_config: Pubkey,
    user_account: Pubkey,
    user_authority: Pubkey,
    credits_to_burn: u32
) -> Instruction {
    Instruction::new_with_borsh(
        crate::id(),
        &ProgramInstruction::UserLevelUp {
            credits_to_burn
        },
        vec![
            AccountMeta::new_readonly(game_config, NOT_A_SIGNER),
            AccountMeta::new(user_account, NOT_A_SIGNER),
            AccountMeta::new_readonly(user_authority, SIGNER)
        ]
    )
}