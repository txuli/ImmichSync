use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug)]
pub struct CheckToken{
    pub valid:bool,
}