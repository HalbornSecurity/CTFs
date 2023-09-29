#![allow(clippy::too_many_arguments)]
use {
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{
        pubkey::{Pubkey},
    },
};

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct Farm {
    pub is_allowed: u8,
    
    pub nonce: u8,

    pub pool_lp_token_account: Pubkey,
    pub pool_reward_token_account: Pubkey,
    pub pool_mint_address: Pubkey,
    pub reward_mint_address: Pubkey,
    pub token_program_id: Pubkey,
    pub owner: Pubkey,
    pub fee_owner: Pubkey,

    pub reward_per_share_net: u64,
    pub last_timestamp: u64,
    pub reward_per_timestamp: u64,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
}

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct Swap {
    pub pool_mint: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey
}