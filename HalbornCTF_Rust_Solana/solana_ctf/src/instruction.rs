#![allow(clippy::too_many_arguments)]

use {
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
};

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
pub enum FarmInstruction {
    ///   Initializes a new Farm.
    ///   These represent the parameters that will be included from client side
    ///   [w] - writable, [s] - signer
    /// 
    ///   0. `[w]` New Farm account to create.
    ///   1. `[]` authority to initialize this farm pool account
    ///   2. `[s]` Creator/Manager of this farm
    ///   3. `[w]` LP token account of this farm to store lp token
    ///   4. `[w]` reward token account of this farm to store rewards for the farmers
    ///             Creator has to transfer/deposit his reward token to this account.
    ///             only support spl tokens
    ///   5. `[]` Pool token mint address
    ///   6. `[]` Reward token mint address
    ///   7. `[]` Amm Id
    ///   8. `[]` Token program id
    ///   9. `[]` nonce
    ///   10. `[]` Farm program id
    ///   11.'[]' start timestamp. this reflects that the farm starts at this time
    ///   12.'[]' end timestamp. this reflects that the farm ends at this time
    Create {
        #[allow(dead_code)]
        /// nonce
        nonce: u8,

        #[allow(dead_code)]
        /// start timestamp
        start_timestamp: u64,

        #[allow(dead_code)]
        /// end timestamp
        end_timestamp: u64,
    },
    
    ///   Creator has to pay farm fee (if not HAL token pairing)
    ///   So this farm can be allowed to stake/unstake/harvest
    /// 
    ///   0. `[w]` Farm to pay farm fee.
    ///   1. `[]` authority of this farm pool
    ///   2. `[s]` payer
    ///   3. `[w]` User transfer authority.
    ///   4. `[]` User CRP token account
    ///   5. `[]` Fee Owner
    ///   6. `[]` Token program id
    ///   7. `[]` Farm program id
    ///   8. `[]` amount
    PayFarmFee(u64),
}

pub fn ix_pay_create_fee(
    farm_id: &Pubkey,
    authority: &Pubkey,
    creator: &Pubkey,
    creator_token_account: &Pubkey,
    fee_vault: &Pubkey,
    token_program_id: &Pubkey,
    farm_program_id: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*farm_id, false),
        AccountMeta::new_readonly(*authority, false),
        AccountMeta::new(*creator, true),
        AccountMeta::new(*creator_token_account, false),
        AccountMeta::new(*fee_vault, false),
        AccountMeta::new_readonly(*token_program_id, false),
    ];
    Instruction {
        program_id: *farm_program_id,
        accounts,
        data: FarmInstruction::PayFarmFee(amount).try_to_vec().unwrap(),
    }
}