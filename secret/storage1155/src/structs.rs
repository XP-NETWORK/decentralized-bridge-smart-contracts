// In general, data that is stored for user display may be different from the data used
// for internal functions of the smart contract. That is why we have StoreOffspringInfo.

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
