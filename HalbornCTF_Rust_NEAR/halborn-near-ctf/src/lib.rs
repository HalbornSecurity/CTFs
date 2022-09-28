use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::ValidAccountId;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, log, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, PromiseOrValue,
};
use std::convert::From;
use std::convert::TryFrom;

near_sdk::setup_alloc!();

pub const GAS_FOR_REGISTER: Gas = 10_000_000_000_000;

#[ext_contract]
pub trait AssociatedContractInterface {
    fn register_for_an_event(&mut self, event_id: U128, account_id: AccountId);
}

#[derive(
    BorshDeserialize, BorshSerialize, Clone, Copy, Eq, PartialEq, Debug, Serialize, Deserialize,
)]
#[serde(crate = "near_sdk::serde")]
pub enum BlocklistStatus {
    Allowed,
    Banned,
}

#[derive(
    BorshDeserialize, BorshSerialize, Clone, Copy, Eq, PartialEq, Debug, Serialize, Deserialize,
)]
#[serde(crate = "near_sdk::serde")]
pub enum ContractStatus {
    Working,
    Paused,
}

impl std::fmt::Display for ContractStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractStatus::Working => write!(f, "working"),
            ContractStatus::Paused => write!(f, "paused"),
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MalbornClubContract {
    owner_id: AccountId,
    malborn_token: FungibleToken,
    token_metadata: LazyOption<FungibleTokenMetadata>,
    block_list: LookupMap<AccountId, BlocklistStatus>,
    status: ContractStatus,
    associated_contract_account_id: LazyOption<AccountId>,
    registration_fee_denominator: U128,
}

#[near_bindgen]
impl MalbornClubContract {
    #[init]
    pub fn new(owner_id: AccountId, token_total_supply: U128) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let metadata = FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "Malborn Token".to_string(),
            symbol: "MAL".to_string(),
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: 10,
        };

        // default registration_fee_denominator results in user burning
        // 0.01% of current total supply to register
        let mut this_state = Self {
            owner_id: owner_id.clone(),
            malborn_token: FungibleToken::new(b"t".to_vec()),
            token_metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
            block_list: LookupMap::new(b"b".to_vec()),
            status: ContractStatus::Working,
            associated_contract_account_id: LazyOption::new(b"a".to_vec(), None),
            registration_fee_denominator: U128::from(10000),
        };
        this_state
            .malborn_token
            .internal_register_account(&owner_id);
        this_state
            .malborn_token
            .internal_deposit(&owner_id, token_total_supply.into());
        this_state
    }

    // Mint tokens to someone. Returns the new total_supply
    pub fn mint_tokens(&mut self, account_id: &AccountId, amount: U128) -> Balance {
        self.only_owner();
        self.not_paused();

        self.malborn_token.total_supply = self
            .malborn_token
            .total_supply
            .checked_add(u128::from(amount))
            .expect("Minting caused overflow");

        if let Some(user_amount) = self.malborn_token.accounts.get(account_id) {
            self.malborn_token.accounts.insert(
                account_id,
                &user_amount
                    .checked_add(u128::from(amount))
                    .expect("Exceeded balance"),
            );
        }

        self.malborn_token.total_supply
    }

    // Burn someone's tokens
    pub fn burn_tokens(&mut self, account_id: &AccountId, amount: U128) {
        self.only_owner();
        self.not_paused();
        self.burn_tokens_internal(account_id, amount);
    }

    // Register to an event in the associated contract
    // User, as part of the MalbornClub, can register for burning some of own tokens
    // This is "access to event for the price of influence over MalbornClub" mechanism.
    pub fn register_for_event(&mut self, event_id: U128) {
        self.not_paused();
        assert!(
            self.associated_contract_account_id.is_some(),
            "Associated Account is not set"
        );
        let sender_id = AccountId::try_from(env::signer_account_id())
            .expect("Couldn't validate sender address");
        self.not_banned(sender_id.clone());

        // burn tokens for registering
        let burn_amount = u128::from(self.malborn_token.total_supply)
            / u128::from(self.registration_fee_denominator);
        self.burn_tokens_internal(&sender_id, U128::from(burn_amount));

        associated_contract_interface::register_for_an_event(
            event_id,
            sender_id,
            &self.associated_contract_account_id.get().unwrap(),
            0,
            GAS_FOR_REGISTER,
        );
    }

    pub fn upgrade_token_name_symbol(&mut self, name: String, symbol: String) {
        self.only_owner();
        let metadata = self.token_metadata.take();
        if let Some(mut metadata) = metadata {
            metadata.name = name;
            metadata.symbol = symbol;
            self.token_metadata.replace(&metadata);
        }
    }

    pub fn add_to_blocklist(&mut self, account_id: &AccountId) {
        self.only_owner();
        self.not_paused();
        self.block_list.insert(account_id, &BlocklistStatus::Banned);
    }

    pub fn remove_from_blocklist(&mut self, account_id: &AccountId) {
        self.only_owner();
        self.not_paused();
        self.block_list
            .insert(account_id, &BlocklistStatus::Allowed);
    }

    pub fn pause(&mut self) {
        self.only_owner();
        self.status = ContractStatus::Paused;
    }

    pub fn resume(&mut self) {
        self.only_owner();
        self.status = ContractStatus::Paused;
    }

    pub fn set_owner(&mut self, new_owner: AccountId) {
        self.only_owner();
        self.owner_id = new_owner;
    }

    pub fn set_registration_fee_denominator(&mut self, new_denominator: U128) {
        self.only_owner();
        self.registration_fee_denominator = new_denominator;
    }

    pub fn set_associated_contract(&mut self, account_id: AccountId) {
        self.only_owner();
        self.associated_contract_account_id.set(&account_id);
    }

    pub fn get_symbol(&mut self) -> String {
        self.not_paused();
        let metadata = self.token_metadata.take();
        metadata
            .expect("Unable to retrieve metadata at this moment")
            .symbol
    }

    pub fn get_name(&mut self) -> String {
        self.not_paused();
        let metadata = self.token_metadata.take();
        metadata
            .expect("Unable to retrieve metadata at this moment")
            .name
    }

    pub fn get_decimals(&mut self) -> u8 {
        self.not_paused();
        let metadata = self.token_metadata.take();
        metadata
            .expect("Unable to retrieve metadata at this moment")
            .decimals
    }

    pub fn contract_status(&self) -> ContractStatus {
        self.status
    }

    pub fn get_blocklist_status(&self, account_id: &AccountId) -> BlocklistStatus {
        self.not_paused();
        return match self.block_list.get(account_id) {
            Some(user_status) => user_status.clone(),
            None => BlocklistStatus::Allowed,
        };
    }

    // **** Helpers ****

    fn burn_tokens_internal(&mut self, account_id: &AccountId, amount: U128) {
        assert!(&self.malborn_token.total_supply >= &Balance::from(amount));
        let user_balance = self
            .malborn_token
            .accounts
            .get(account_id)
            .expect("User not registered");
        assert!(user_balance >= u128::from(amount));

        self.malborn_token.total_supply = self
            .malborn_token
            .total_supply
            .checked_sub(u128::from(amount))
            .expect("Burn caused underflow");

        self.malborn_token.accounts.insert(
            account_id,
            &user_balance
                .checked_sub(u128::from(amount))
                .expect("Underflow in user balance"),
        );
    }

    fn only_owner(&self) {
        if env::signer_account_id() != self.owner_id {
            env::panic("Can only be called by owner".as_bytes());
        }
    }

    fn not_paused(&self) {
        if self.status == ContractStatus::Paused {
            env::panic("Contract is paused".as_bytes());
        }
    }

    fn not_banned(&self, account_id: AccountId) {
        if self.get_blocklist_status(&account_id) == BlocklistStatus::Banned {
            env::panic("User is banned".as_bytes());
        }
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }
}

#[near_bindgen]
impl FungibleTokenCore for MalbornClubContract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: ValidAccountId, amount: U128, memo: Option<String>) {
        self.not_paused();
        let sender_id = AccountId::try_from(env::signer_account_id())
            .expect("Couldn't validate sender address");
        self.not_banned(sender_id.clone());
        assert!(
            u128::from(amount)
                <= u128::from(self.ft_balance_of(ValidAccountId::try_from(sender_id).unwrap()))
        );
        self.malborn_token
            .ft_transfer(receiver_id.clone(), amount, memo);
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: ValidAccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.not_paused();
        let sender_id = AccountId::try_from(env::signer_account_id())
            .expect("Couldn't validate sender address");
        self.not_banned(sender_id.clone());
        self.malborn_token
            .ft_transfer_call(receiver_id.clone(), amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.not_paused();
        self.malborn_token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: ValidAccountId) -> U128 {
        self.not_paused();
        self.malborn_token
            .ft_balance_of(ValidAccountId::try_from(account_id).unwrap())
    }
}

#[near_bindgen]
impl FungibleTokenResolver for MalbornClubContract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: ValidAccountId,
        receiver_id: ValidAccountId,
        amount: U128,
    ) -> U128 {
        self.malborn_token
            .internal_ft_resolve_transfer(&sender_id.to_string(), receiver_id, amount)
            .0
            .into()
    }
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for MalbornClubContract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.token_metadata.get().unwrap()
    }
}

near_contract_standards::impl_fungible_token_storage!(
    MalbornClubContract,
    malborn_token,
    on_account_closed
);

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    const TOTAL_SUPPLY: Balance = 1_000_000_000;

    fn get_context(
        predecessor_account_id: AccountId,
        signer_account_id: AccountId,
    ) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(
                near_sdk::json_types::ValidAccountId::try_from(signer_account_id).unwrap(),
            )
            .predecessor_account_id(
                near_sdk::json_types::ValidAccountId::try_from(predecessor_account_id).unwrap(),
            );
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1).to_string(), accounts(2).to_string());
        testing_env!(context.build());
        let contract = MalbornClubContract::new(accounts(1).into(), TOTAL_SUPPLY.into());
        testing_env!(context.is_view(true).build());

        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(
            contract.ft_balance_of(accounts(1)),
            U128::from(TOTAL_SUPPLY)
        );
    }

    #[test]
    fn test_mint() {
        let context = get_context(accounts(2).to_string(), accounts(2).to_string());
        testing_env!(context.build());
        let mut contract = MalbornClubContract::new(accounts(2).into(), TOTAL_SUPPLY.into());

        let mint_amount = TOTAL_SUPPLY / 2;

        contract.mint_tokens(&accounts(2).to_string(), U128::from(mint_amount));
        assert_eq!(
            contract.ft_balance_of(accounts(2)).0,
            TOTAL_SUPPLY + mint_amount
        );
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY + mint_amount);
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2).to_string(), accounts(2).to_string());
        testing_env!(context.build());
        let mut contract = MalbornClubContract::new(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration => storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(
            contract.ft_balance_of(accounts(2)).0,
            (TOTAL_SUPPLY - transfer_amount)
        );
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }

    #[test]
    #[should_panic]
    fn test_pause() {
        let context = get_context(accounts(2).to_string(), accounts(2).to_string());
        testing_env!(context.build());
        let mut contract = MalbornClubContract::new(accounts(2).into(), TOTAL_SUPPLY.into());

        let symbol = contract.get_symbol();
        assert_eq!(symbol, "MAL".to_string());
        contract.pause();
        contract.get_symbol();
    }

    #[test]
    fn test_blocklist() {
        let context = get_context(accounts(2).to_string(), accounts(2).to_string());
        testing_env!(context.build());
        let mut contract = MalbornClubContract::new(accounts(2).into(), TOTAL_SUPPLY.into());
        assert_eq!(
            contract.get_blocklist_status(&accounts(1).to_string()),
            BlocklistStatus::Allowed
        );

        contract.add_to_blocklist(&accounts(1).to_string());
        assert_eq!(
            contract.get_blocklist_status(&accounts(1).to_string()),
            BlocklistStatus::Banned
        );

        contract.remove_from_blocklist(&accounts(1).to_string());
        assert_eq!(
            contract.get_blocklist_status(&accounts(1).to_string()),
            BlocklistStatus::Allowed
        );
    }

    #[test]
    #[should_panic]
    fn test_blocklist2() {
        let mut context = get_context(accounts(2).to_string(), accounts(2).to_string());
        testing_env!(context.build());
        let mut contract = MalbornClubContract::new(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration -- storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        assert_eq!(
            contract.get_blocklist_status(&accounts(1).to_string()),
            BlocklistStatus::Allowed
        );

        contract.add_to_blocklist(&accounts(1).to_string());
        assert_eq!(
            contract.get_blocklist_status(&accounts(1).to_string()),
            BlocklistStatus::Banned
        );

        testing_env!(context
            .predecessor_account_id(accounts(1))
            .signer_account_id(accounts(1))
            .build());

        contract.ft_transfer(accounts(2), U128::from(transfer_amount / 2), None);
    }
}
