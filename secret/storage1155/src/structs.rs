use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Info needed to instantiate an offspring
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct CodeInfo {
    /// code id of the stored offspring contract
    pub code_id: u64,
    /// code hash of the stored offspring contract
    pub code_hash: String,
}
