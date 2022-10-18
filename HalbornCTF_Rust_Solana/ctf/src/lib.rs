use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo,
    declare_id,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

pub mod state;
pub mod instructions;
pub mod constants;
mod processor;

use instructions::ProgramInstruction;
use processor::*;

declare_id!("GAME8ZGUzNChyRXHMxR4fVTvhpNDa6dJyK8oVmydp4RZ");

entrypoint!(process_instruction);

/// Top level instruction processor
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match ProgramInstruction::try_from_slice(instruction_data)? {
        ProgramInstruction::CreateGameConfig { credits_per_level } => create_game_config(credits_per_level, accounts),
        ProgramInstruction::CreateUser { } => create_user(accounts),
        ProgramInstruction::MintCreditsToUser { credits } => mint_credits_to_user(credits, accounts),
        ProgramInstruction::UserLevelUp { credits_to_burn } => user_level_up(credits_to_burn, accounts)
    }
}