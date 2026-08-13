use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug)]
pub struct CheckToken {
    pub valid: bool,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct ValidResponse {
    pub valid: bool,
    pub type_acc: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Settings {
    pub url: String,
    pub token: String,
}
