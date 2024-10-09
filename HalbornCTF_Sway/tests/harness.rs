use fuels::{prelude::*, types::ContractId,types::Bytes32, types::{AssetId, Identity},types::bech32::Bech32ContractId};
use sha2::{Digest, Sha256};
use std::str::FromStr;

// Load ABI from JSON
abigen!(Contract(
    name = "SimpleLiquidityPool",
    abi = "out/debug/SimpleLiquidityPool-abi.json"
));


// helper function to get an asset ID given a subId and a contractId
pub fn get_asset_id(sub_id: Bytes32, contract: ContractId) -> AssetId {
    let mut hasher = Sha256::new();
    hasher.update(*contract);
    hasher.update(*sub_id);
    AssetId::new(*Bytes32::from(<[u8; 32]>::from(hasher.finalize())))
}

// helper function to get the default assetId for a given contract
pub fn get_default_asset_id(contract: ContractId) -> AssetId {
    let default_sub_id = Bytes32::from([0u8; 32]);
    get_asset_id(default_sub_id, contract)
}

/// THIS FUNCTION DOES NOT HAVE TO BE MODIFIED ///
async fn get_contract_instance(
    wallet_admin: WalletUnlocked,
    wallet_attacker: WalletUnlocked,
    provider: Provider,
    base_asset_id: AssetId
) -> (
    SimpleLiquidityPool<WalletUnlocked>,
    ContractId,
) {

    // Deploy the contract using wallet_admin
    let id = Contract::load_from(
        "./out/debug/simpleLiquidityPool.bin", // Adjust the path
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet_admin.clone(), TxPolicies::default())
    .await
    .unwrap();

    let instance = SimpleLiquidityPool::new(id.clone(), wallet_attacker.clone());

    let funding_amounts = vec![1_000_000,1_000_000,1_000_000, 2_000_000, 3_000_000,2_000_000]; // Total: 10,000,000

    for amount in funding_amounts {
        wallet_admin
            .force_transfer_to_contract(
                &id,
                amount,
                base_asset_id,
                TxPolicies::default(),
            )
            .await
            .unwrap();
    }

    let contract_base_token_balance = provider
        .get_contract_asset_balance(&id, base_asset_id)
        .await
        .unwrap();
    assert_eq!(contract_base_token_balance, 10_000_000);

    (instance, id.into())
}


/// THIS FUNCTION CAN BE MODIFIED => CODE EXPLOIT HERE ///
#[tokio::test]
async fn test_liquidity_pool(
) {
    let base_asset_id: AssetId =
        "0x9ae5b658754e096e4d681c548daf46354495a437cc61492599e33fc64dABCD12"
            .parse()
            .unwrap();

    // Create two wallets with minimal initial assets
    let mut wallet_admin = WalletUnlocked::new_random(None);
    let mut wallet_attacker = WalletUnlocked::new_random(None);

    // Create AssetConfigs for wallet_admin
    let admin_asset_configs = vec![
        AssetConfig {
            id: AssetId::default(), // Default asset for gas
            num_coins: 1,
            coin_amount: 10_000_000,
        },
        AssetConfig {
            id: base_asset_id,
            num_coins: 1,
            coin_amount: 10_000_000,
        },
    ];

    // Create coins for wallet_admin
    let admin_coins = setup_custom_assets_coins(
        wallet_admin.address(),
        &admin_asset_configs,
    );

    let attacker_asset_configs = vec![
        AssetConfig {
            id: AssetId::default(), // Default asset for gas
            num_coins: 1,
            coin_amount: 2_000,
        },
    ];

    let attacker_coins = setup_custom_assets_coins(
        wallet_attacker.address(),
        &attacker_asset_configs,
    );

    // Combine all coins
    let coins = [admin_coins, attacker_coins].concat();

    // Launch provider with coins
    // Launch provider with coins
    let provider = setup_test_provider(
        coins,
        vec![], // No messages
        None,   // No custom Config
        None,   // No ChainConfig
    )
    .await
    .unwrap();

    // Assign provider to wallets
    wallet_admin.set_provider(provider.clone());
    wallet_attacker.set_provider(provider.clone());

    
    let (
        simple_liquidity_pool_instance, 
        contract_id,
    ) = get_contract_instance(wallet_admin,wallet_attacker.clone(),provider,base_asset_id).await;

    let lp_token_sub_id = Bytes32::from_str("0x000000000000000000000000000000000000000000000000000DEAD000123456").unwrap();
    let lp_token_asset_id = get_asset_id(lp_token_sub_id,contract_id.clone());

    let bech32_contract_id = Bech32ContractId::from(contract_id);


    // Check initial BASE_TOKEN balance
    println!("---- INITIAL ----\n");
    let initial_base_balance = wallet_attacker.get_asset_balance(&base_asset_id).await.unwrap();
    let initial_native_token = wallet_attacker.get_asset_balance(&AssetId::zeroed()).await.unwrap();
    assert_eq!(initial_native_token, 2_000);

    let contract_base_token_balance = wallet_attacker.provider().unwrap()
        .get_contract_asset_balance(&bech32_contract_id, base_asset_id).await.unwrap();

    println!("Attacker Initial base_token balance:          {}", initial_base_balance);
    println!("----");
    println!("Contract Initial base_token balance:          {}",contract_base_token_balance);

    println!("\n---- EXPLOIT ----\n");




    /* BONNE CHANCE
    
    
    
    
    
    





    








    BONNE CHANCE */



    // ----- DO NOT MODIFY ----- //

    // Check final BASE_TOKEN balance

    let final_base_balance = wallet_attacker.get_asset_balance(&base_asset_id).await.unwrap();
    
    let contract_base_token_balance_final = wallet_attacker.provider().unwrap().get_contract_asset_balance(&bech32_contract_id, base_asset_id)
        .await.unwrap();

    println!("Attacker Final base_token balance:            {}", final_base_balance);
    println!("----");
    println!("Contract Final base_token balance:            {}",contract_base_token_balance_final);

    println!("Exploit successful Benefits =                 {} ",final_base_balance - initial_base_balance);


    // THESE ASSERTIONS SHOULD PASS
    assert_eq!(final_base_balance, 10_000_000);
    assert_eq!(contract_base_token_balance_final, 0);

    // ----- DO NOT MODIFY ----- //
    
}
