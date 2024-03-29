use cosmwasm_schema::write_api;

use nft_store::msg::{NftStoreExecuteMsg, NftStoreInstantiateMsg};

fn main() {
    write_api! {
        instantiate: NftStoreInstantiateMsg,
        execute: NftStoreExecuteMsg,
    }
}
