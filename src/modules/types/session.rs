use std::time::Instant;
use tokio::sync::Mutex;
use crate::modules::implements::kvstore::KVStore;

pub struct Session {
    pub(crate) store: Mutex<Option<KVStore>>,
    pub(crate) last_active: Mutex<Instant>,
    pub(crate) current_path: Mutex<Option<String>>,
}
