use std::error::Error;

use bridge::types::{AddValidator, BlacklistValidator};
use ed25519_dalek::{ed25519::signature::SignerMut, Keypair, PublicKey};
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

async fn create_validator_aid_pair(sandbox: &mut Worker<Sandbox>) -> (AccountId, Keypair) {
    let mut rng = rand::thread_rng();
    let kp = Keypair::generate(&mut rng);
    let aid = sandbox.dev_create_account().await.unwrap();
    (aid.id().clone(), kp)
}

#[test]
async fn add_validator() -> Result<(), Box<dyn Error>> {
    let mut sandbox = near_workspaces::sandbox().await?;
    let admin = sandbox.dev_create_account().await?;
    let mut rng = rand::thread_rng();
    let mut bootstrap_validator = ed25519_dalek::Keypair::generate(&mut rng);

    let (_, _, bridge) =
        initialize_bridge(&mut sandbox, &admin, bootstrap_validator.public).await?;

    let (nv_aid, nv_kp) = create_validator_aid_pair(&mut sandbox).await;

    let msg = AddValidator {
        account_id: nv_aid.clone(),
        public_key: hex::encode(nv_kp.public.to_bytes()),
    };

    let add = admin
        .call(bridge.id(), "add_validator")
        .args_json(json!({
            "validator": msg,
            "signatures": [{
                "signer": hex::encode(bootstrap_validator.public.to_bytes()),
                "signature": bootstrap_validator.sign(&near_sdk::borsh::to_vec(&msg).unwrap()).to_bytes().to_vec()
            }]
        }))
        .transact()
        .await?;
    eprintln!("{:#?}", add);
    assert!(add.is_success());
    Ok(())
}

#[test]
async fn blacklist_validator() -> Result<(), Box<dyn Error>> {
    let mut sandbox = near_workspaces::sandbox().await?;
    let admin = sandbox.dev_create_account().await?;
    let mut rng = rand::thread_rng();
    let mut bootstrap_validator = ed25519_dalek::Keypair::generate(&mut rng);

    let (_, _, bridge) =
        initialize_bridge(&mut sandbox, &admin, bootstrap_validator.public).await?;

    let (nv_aid, mut nv_kp) = create_validator_aid_pair(&mut sandbox).await;

    let msg = AddValidator {
        account_id: nv_aid.clone(),
        public_key: hex::encode(nv_kp.public.to_bytes()),
    };

    let add = admin
        .call(bridge.id(), "add_validator")
        .args_json(json!({
            "validator": msg,
            "signatures": [{
                "signer": hex::encode(bootstrap_validator.public.to_bytes()),
                "signature": bootstrap_validator.sign(&near_sdk::borsh::to_vec(&msg).unwrap()).to_bytes().to_vec()
            }]
        }))
        .transact()
        .await?;
    assert!(add.is_success());

    let bmsg = BlacklistValidator {
        public_key: hex::encode(nv_kp.public.to_bytes()),
    };

    let remove = admin
        .call(bridge.id(), "blacklist_validator")
        .args_json(json!({
            "validator": bmsg,
            "signatures": [{
                "signer": hex::encode(bootstrap_validator.public.to_bytes()),
                "signature": bootstrap_validator.sign(&near_sdk::borsh::to_vec(&bmsg).unwrap()).to_bytes().to_vec()
            }, {
                "signer": hex::encode(nv_kp.public.to_bytes()),
                "signature": nv_kp.sign(&near_sdk::borsh::to_vec(&bmsg).unwrap()).to_bytes().to_vec()
            }]
        }))
        .transact()
        .await?;
    eprintln!("{:#?}", remove);
    assert!(remove.is_success());
    Ok(())
}
