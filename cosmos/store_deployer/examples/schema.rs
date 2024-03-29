use cosmwasm_schema::write_api;

use store_deployer::msg::{StoreFactoryExecuteMsg, StoreFactoryInstantiateMsg};

fn main() {
    write_api! {
        instantiate: StoreFactoryInstantiateMsg,
        execute: StoreFactoryExecuteMsg,
    }
}
