use std::error::Error;

use bridge::types::{AddValidator, BlacklistValidator, ClaimData};
use ed25519_dalek::{ed25519::signature::SignerMut, Keypair, PublicKey};
use helpers::{approve_nft, mint_nft};
use near_sdk::AccountId;
use near_workspaces::{network::Sandbox, types::NearToken, Account, Contract, Worker};
use serde_json::json;
use tokio::test;

mod helpers;

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

#[test]
async fn lock_nft() -> Result<(), Box<dyn Error>> {
    let mut sandbox = near_workspaces::sandbox().await?;
    let admin = sandbox.dev_create_account().await?;
    let mut rng = rand::thread_rng();
    let bootstrap_validator = ed25519_dalek::Keypair::generate(&mut rng);

    let (_, _, bridge) =
        initialize_bridge(&mut sandbox, &admin, bootstrap_validator.public).await?;

    let nft = near_workspaces::compile_project("../nft/.").await?;
    let nft = sandbox.dev_deploy(&nft).await?;
    let intialize_nft = admin
        .call(nft.id(), "new_default_meta")
        .args_json(json!({"owner_id": admin.id()}))
        .transact()
        .await?;
    assert!(intialize_nft.is_success());

    mint_nft(&admin, &nft, "token-1").await?;

    approve_nft(&bridge, &admin, &nft, "token-1").await?;

    let lock = admin
        .call(bridge.id(), "lock_nft")
        .max_gas()
        .args_json(json!({
            "source_nft_contract_address": nft.id(),
            "token_id": "token-1",
            "destination_chain": "BSC",
            "destination_address": "0x1234567890123456789012345678901234567890",
        }))
        .transact()
        .await?;
    eprintln!("{:#?}", lock);
    assert!(lock.is_success());
    Ok(())
}

#[test]
async fn claim_nft() -> Result<(), Box<dyn Error>> {
    let mut sandbox = near_workspaces::sandbox().await?;
    let admin = sandbox.dev_create_account().await?;
    let mut rng = rand::thread_rng();
    let mut bootstrap_validator = ed25519_dalek::Keypair::generate(&mut rng);

    let (_, _, bridge) =
        initialize_bridge(&mut sandbox, &admin, bootstrap_validator.public).await?;

    let cd = ClaimData {
        destination_chain: "NEAR".to_string(),
        source_chain: "BSC".to_string(),
        destination_user_address: admin.id().clone(),
        token_id: "token-1".to_string(),
        source_nft_contract_address: "nft".to_string(),
        lock_tx_chain: "BSC".to_string(),
        name: "Grumpy Cat".to_string(),
        symbol: "GC".to_string(),
        royalty: 0,
        royalty_receiver: admin.id().clone(),
        metadata: "https://www.adamsdrafting.com/wp-content/uploads/2018/06/More-Grumpy-Cat.jpg"
            .to_string(),
        transaction_hash: "0x1234567890123456789012345678901234567890".to_string(),
        token_amount: 1,
        nft_type: "singular".to_string(),
        fee: NearToken::from_near(1).as_yoctonear().into(),
    };

    let claim = admin.call(bridge.id(), "claim_nft")
    .deposit(NearToken::from_yoctonear(cd.fee.clone().into()))
    .max_gas()
    .args_json(json!({
        "cd": cd,
        "signatures": [
            {
                "signer": hex::encode(bootstrap_validator.public.to_bytes()),
                "signature": bootstrap_validator.sign(&near_sdk::borsh::to_vec(&cd).unwrap()).to_bytes().to_vec()
            }
        ]
    })).transact().await?;

    eprintln!("{:#?}", claim);
    assert!(claim.is_success());
    Ok(())
}
