use bridge::{
    msg::{BridgeExecuteMsg, BridgeQueryMsg},
    structs::BridgeInstantiateMsg,
};
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: BridgeInstantiateMsg,
        execute: BridgeExecuteMsg,
        query: BridgeQueryMsg
    }
}
