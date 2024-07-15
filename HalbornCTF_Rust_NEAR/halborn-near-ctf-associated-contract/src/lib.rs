use near_sdk::borsh::{BorshSerialize, BorshDeserialize};
use near_sdk::collections::{LookupMap, LookupSet};
use near_sdk::json_types::U64;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, log, near_bindgen, AccountId, PanicOnDefault};

mod storage;

use storage::StorageKey;

#[derive(
    BorshDeserialize, BorshSerialize, Clone, Copy, Eq, PartialEq, Debug, Serialize, Deserialize,
)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub enum ContractStatus {
    Working,
    Paused,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct Event {
    created_timestamp: u64,
    is_live: bool,
    title: Option<String>,
}

impl std::fmt::Display for ContractStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractStatus::Working => write!(f, "working"),
            ContractStatus::Paused => write!(f, "paused"),
        }
    }
}

type RegisteredUsers = LookupSet<AccountId>;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct AssociatedContract {
    privileged_clubs: LookupSet<AccountId>,
    events: LookupMap<U64, Event>,
    event_to_registered_users: LookupMap<U64, RegisteredUsers>,
    owner_id: AccountId,
    next_event_idx: u16,
    status: ContractStatus,
}

#[near_bindgen]
impl AssociatedContract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            privileged_clubs: LookupSet::new(StorageKey::PrivilegedClubs.into_bytes()),
            events: LookupMap::new(StorageKey::EventsOnline.into_bytes()),
            event_to_registered_users: LookupMap::new(
                StorageKey::EventToRegisteredUsers.into_bytes(),
            ),
            owner_id: owner_id,
            next_event_idx: 1,
            status: ContractStatus::Working,
        }
    }

    pub fn get_next_event_idx(&self) -> u16 {
        self.next_event_idx
    }

    pub fn get_event(&self, event_idx: U64) -> Event {
        self.events.get(&event_idx).unwrap()
    }

    // Adds new event. Adds new event's id to LookupSet and creates a LookupMap
    // for checking which user registered for that event. Anyone can add an event.
    pub fn add_new_event(&mut self, event_title: String) -> u16 {
        let new_event = Event {
            created_timestamp: env::block_timestamp(),
            is_live: true,
            title: Some(event_title),
        };
        self.events
            .insert(&U64::from(u64::from(self.next_event_idx)), &new_event);

        let new_ev_to_registered_users: RegisteredUsers =
            LookupSet::new(StorageKey::RegisteredUsers(self.next_event_idx).into_bytes());

        self.event_to_registered_users.insert(
            &U64::from(u64::from(self.next_event_idx)),
            &new_ev_to_registered_users,
        );
        let created_event_idx = self.next_event_idx;
        self.next_event_idx += 1;
        created_event_idx
    }

    pub fn remove_event(&mut self, event_id: U64) {
        self.only_owner();
        self.events.remove(&event_id);
        self.event_to_registered_users.remove(&event_id);
    }

    // If the event is over, or was cancelled, or whatever the reason
    // we delete it from events_online. We leave the event_to_registered_users intact
    // because we still might want to see who was registered for previous events
    pub fn make_event_offline(&mut self, event_id: U64) {
        self.only_owner();
        self.events.get(&event_id).unwrap().is_live = false;
    }

    pub fn add_privileged_club(&mut self, account_id: AccountId) {
        self.only_owner();
        self.privileged_clubs.insert(&account_id);
    }

    pub fn remove_privileged_club(&mut self, account_id: AccountId) {
        self.only_owner();
        self.privileged_clubs.remove(&account_id);
    }

    pub fn register_for_an_event(&mut self, event_id: U64, account_id: AccountId) {
        self.only_from_privileged_club();
        assert!(self.events.contains_key(&event_id), "No event with such ID");
        assert!(
            self.events.get(&event_id).unwrap().is_live,
            "Event is no longer live"
        );

        self.event_to_registered_users
            .get(&event_id)
            .unwrap()
            .insert(&account_id);
        log!(
            "{} registered for event: {}",
            account_id,
            u64::from(event_id)
        );
    }

    pub fn check_user_registered(&self, event_id: U64, account_id: AccountId) -> bool {
        self.events.contains_key(&event_id)
            && self
                .event_to_registered_users
                .get(&event_id)
                .unwrap()
                .contains(&account_id)
    }

    pub fn set_owner(&mut self, account_id: AccountId) {
        self.only_owner();
        self.owner_id = account_id;
    }

    pub fn pause(&mut self) {
        self.only_owner();
        self.status = ContractStatus::Paused;
    }

    pub fn resume(&mut self) {
        self.only_owner();
        self.status = ContractStatus::Working;
    }

    fn only_from_privileged_club(&self) {
        if !self
            .privileged_clubs
            .contains(&env::predecessor_account_id())
        {
            env::panic_str("Can be called only from a privileged club contract");
        }
    }

    fn only_owner(&self) {
        if env::signer_account_id() != self.owner_id {
            env::panic_str("Only owner can call this function");
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    fn get_context(
        predecessor_account_id: AccountId,
        signer_account_id: AccountId,
    ) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(
                signer_account_id,
            )
            .predecessor_account_id(
                predecessor_account_id,
            );
        builder
    }

    #[test]
    fn add_events() {
        let context = get_context(accounts(1), accounts(1));
        testing_env!(context.build());

        let mut contract = AssociatedContract::new(accounts(1).into());
        contract.add_new_event("Event1".to_string());
        contract.add_new_event("Event2".to_string());
        contract.add_new_event("Event3".to_string());

        assert!(contract.events.contains_key(&U64::from(3)));
        assert_eq!(contract.next_event_idx, 4);
    }

    #[test]
    fn add_event_and_register_users() {
        let mut context = get_context(accounts(1), accounts(1));
        testing_env!(context.build());

        let mut contract = AssociatedContract::new(accounts(1).into());
        contract.add_privileged_club(accounts(2));
        contract.add_new_event("Event1".to_string());

        assert_eq!(contract.next_event_idx, 2);

        testing_env!(context
            .predecessor_account_id(accounts(2))
            .signer_account_id(accounts(2))
            .build());

        contract.register_for_an_event(U64::from(1), accounts(3));
        contract.register_for_an_event(U64::from(1), accounts(4));

        let registered_users = contract
            .event_to_registered_users
            .get(&U64::from(1))
            .unwrap();
        assert!(registered_users.contains(&accounts(3)));
        assert!(registered_users.contains(&accounts(4)));
    }
}
