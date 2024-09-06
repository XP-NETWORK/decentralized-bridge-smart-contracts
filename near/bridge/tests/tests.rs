use std::error::Error;

use ed25519_dalek::{ed25519::signature::SignerMut, PublicKey};
use near_sdk::AccountId;
use near_workspaces::{network::Sandbox, Account, Contract, Worker};
use serde_json::json;
use tokio::test;

async fn initialize_bridge(
    sandbox: &mut Worker<Sandbox>,
    admin: &Account,
    pk: PublicKey,
) -> Result<(Contract, Contract, Contract), Box<dyn Error>> {
    let bridge = near_workspaces::compile_project("./").await?;
    let collection_factory = near_workspaces::compile_project("../collection-factory/.").await?;
    let storage_factory = near_workspaces::compile_project("../storage-factory/.").await?;
    let collection_factory = sandbox.dev_deploy(&collection_factory).await?;
    let storage_factory = sandbox.dev_deploy(&storage_factory).await?;
    let bridge = sandbox.dev_deploy(&bridge).await?;

    let init_sf = admin
        .call(storage_factory.id(), "new")
        .args_json(json!({
            "owner": bridge.id(),
        }))
        .transact()
        .await?;
    assert!(init_sf.is_success());

    let init_cf = admin
        .call(collection_factory.id(), "new")
        .args_json(json!({
            "owner": bridge.id(),
        }))
        .transact()
        .await?;
    assert!(init_cf.is_success());

    let init = admin
        .call(bridge.id(), "new")
        .args_json(json!({
            "collection_factory": collection_factory.id(),
            "storage_factory": storage_factory.id(),
            "validators": [
                hex::encode(pk.to_bytes())
            ],
        }))
        .transact()
        .await?;
    assert!(init.is_success());
    Ok((collection_factory, storage_factory, bridge))
}

#[test]
async fn initialize() -> Result<(), Box<dyn Error>> {
    let mut sandbox = near_workspaces::sandbox().await?;
    let admin = sandbox.dev_create_account().await?;
    let bootstrap_validator = ed25519_dalek::Keypair::generate(&mut rand::thread_rng());

    let (collection_factory, storage_factory, bridge) =
        initialize_bridge(&mut sandbox, &admin, bootstrap_validator.public).await?;

    assert!(
        bridge
            .view("validator_count")
            .await?
            .json::<u128>()
            .unwrap()
            == 1
    );

    assert!(
        bridge
            .view("collection_factory")
            .await?
            .json::<AccountId>()
            .unwrap()
            == *collection_factory.id()
    );

    assert!(
        bridge
            .view("storage_factory")
            .await?
            .json::<AccountId>()
            .unwrap()
            == *storage_factory.id()
    );
    Ok(())
}

#[test]
async fn add_validator() -> Result<(), Box<dyn Error>> {
    let mut sandbox = near_workspaces::sandbox().await?;
    let admin = sandbox.dev_create_account().await?;
    let mut rng = rand::thread_rng();
    let mut bootstrap_validator = ed25519_dalek::Keypair::generate(&mut rng);

    let nv = ed25519_dalek::Keypair::generate(&mut rng);

    let (_, _, bridge) =
        initialize_bridge(&mut sandbox, &admin, bootstrap_validator.public).await?;

    let add = admin
        .call(bridge.id(), "add_validator")
        .args_json(json!({
            "validator": hex::encode(nv.public.to_bytes()),
            "signatures": [{
                "signer": bootstrap_validator.public.to_bytes().to_vec(),
                "signature": bootstrap_validator.sign(&nv.public.to_bytes()).to_bytes().to_vec()
            }]
        }))
        .transact()
        .await?;
    eprintln!("{:#?}", add);
    assert!(add.is_success());
    Ok(())
}
