contract;

use std::{
    asset::{mint_to,burn,mint, transfer},
    call_frames::msg_asset_id,
    context::{msg_amount},
    auth::msg_sender,
    hash::Hash,
    revert::require,
    contract_id::ContractId,
    constants::ZERO_B256,
};

abi SimpleLiquidityPool {
    #[payable]
    fn deposit_base_asset();
    #[payable]
    #[storage(read, write)]
    fn deposit(recipient: Identity);
    #[payable]
    #[storage(read, write)]
    fn withdraw(recipient: Identity);
}

storage {
    balances: StorageMap<Identity, u64> = StorageMap {},
    total_deposited: u64 = 0,
}

const BASE_TOKEN: AssetId = AssetId::from(0x9ae5b658754e096e4d681c548daf46354495a437cc61492599e33fc64dABCD12);
const LP_TOKEN_SUB_ID = 0x000000000000000000000000000000000000000000000000000DEAD000123456;


impl SimpleLiquidityPool for Contract {

    #[payable]
    fn deposit_base_asset() {
        let amount = msg_amount();
    }

    #[payable]
    #[storage(read, write)]
    fn deposit(recipient: Identity) {
        let amount = msg_amount();
        let sender = msg_sender().unwrap();
        
        let current_balance = storage.balances.get(sender).try_read().unwrap_or(0);

        storage.balances.insert(sender, current_balance + amount);

        let total_deposited = storage.total_deposited.read();
        storage.total_deposited.write(total_deposited + amount);

        // Mint 4 times the amount in LP TOKENS
        let amount_to_mint = msg_amount() * 4;

        mint_to(recipient,LP_TOKEN_SUB_ID, amount_to_mint);

    }

    #[payable]
    #[storage(read, write)]
    fn withdraw(recipient: Identity) {

        assert(AssetId::new(ContractId::this(), LP_TOKEN_SUB_ID) == msg_asset_id());

        assert(0 < msg_amount());
        
        let amount = msg_amount() / 4;
        let sender = msg_sender().unwrap();

        let current_balance = storage.balances.get(sender).try_read().unwrap_or(0);
        
        // Ensure sufficient amount deposited first
        require(current_balance >= amount, "Insufficient balance");
        storage.balances.insert(sender, current_balance - amount);

        let total_deposited = storage.total_deposited.read();
        storage.total_deposited.write(total_deposited - amount);

        // Transfer the base token back to the user
        transfer(recipient,BASE_TOKEN, amount);
    }

}


