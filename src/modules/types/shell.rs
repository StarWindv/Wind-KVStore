use crate::modules::implements::kvstore::KVStore;

pub struct Shell {
    pub(crate) store: Option<KVStore>,
    pub(crate) current_path: Option<String>,
}
