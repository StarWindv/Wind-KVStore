use serde::Deserialize;

#[derive(Deserialize)]
pub struct KeyValueRequest {
    pub(crate) key: String,
    pub(crate) value: Option<String>,
}
