use serde::Serialize;

#[derive(Serialize)]
pub struct StatusResponse {
    pub(crate) status: String,
}
