use cosm_nft::init::InstantiateMsg;
use cosm_nft::{NftExecuteMsg, NftQueryMsg};
use cosmwasm_schema::write_api;
fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: NftExecuteMsg,
        query: NftQueryMsg,
    }
}
