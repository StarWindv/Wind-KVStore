use std::sync::Arc;
use dashmap::DashMap;
use crate::modules::types::server_config::ServerConfig;
use crate::modules::types::session::Session;

pub type SessionManager = Arc<DashMap<String, Arc<Session>>>;

pub struct WindServer {
    pub(crate) config: ServerConfig,
    pub(crate) sessions: SessionManager,
    pub(crate) print_header: bool,
}
