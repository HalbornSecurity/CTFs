pub enum StorageKey {
    PrivilegedClubs,
    EventsOnline,
    EventToRegisteredUsers,
    RegisteredUsers(u16),
}

impl StorageKey {
    pub fn to_string(&self) -> String {
        match self {
            StorageKey::PrivilegedClubs => "priv_clubs".to_string(),
            StorageKey::EventsOnline => "events".to_string(),
            StorageKey::EventToRegisteredUsers => "ev_to_registered".to_string(),
            StorageKey::RegisteredUsers(event_id) => format!("event{}", event_id),
        }
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}
