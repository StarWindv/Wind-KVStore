use serde::Serialize;

#[derive(Serialize)]
pub struct KeyValueResponse {
    pub(crate) key: String,
    pub(crate) value: Option<String>,
}
