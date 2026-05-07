use serde::Deserialize;

#[derive(Deserialize)]
pub struct IdentifierRequest {
    pub(crate) identifier: String,
}
