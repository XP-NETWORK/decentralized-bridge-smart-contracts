use cosmwasm_schema::write_api;

use collection_deployer::msg::{CollectionDeployerExecuteMsg, CollectionDeployerInstantiateMsg};

fn main() {
    write_api! {
        instantiate: CollectionDeployerInstantiateMsg,
        execute: CollectionDeployerExecuteMsg,
    }
}
