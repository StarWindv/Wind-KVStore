use serde::Serialize;

#[derive(Serialize)]
pub struct IdentifierResponse {
    pub(crate) identifier: String,
}
