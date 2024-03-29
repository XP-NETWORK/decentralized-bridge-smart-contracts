use cosmwasm_schema::cw_serde;

#[cw_serde]
#[derive(Default)]
pub struct RoyaltyData {
    /// The percentage. must be between 0-100.
    pub royalty_percentage: u64,
    /// The payment address, may be different to or the same
    /// as the minter addr
    /// question: how do we validate this?
    pub royalty_payment_address: String,
}
