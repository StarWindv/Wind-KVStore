use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct PathRequest {
    pub(crate) path: String,
}
