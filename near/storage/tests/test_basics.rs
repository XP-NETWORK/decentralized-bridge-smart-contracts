use helper::{mint_nft, transfer_nft};
use near_sdk::AccountId;
use serde_json::json;

mod helper;


#[tokio::test]
async fn contract_intialized() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;

    let storage = near_workspaces::compile_project("./").await?;
    let nft = near_workspaces::compile_project("../nft/.").await?;
    let storage = sandbox.dev_deploy(&storage).await?;
    let nft = sandbox.dev_deploy(&nft).await?;

    let user_account = sandbox.dev_create_account().await?;

    let initialize_storage = user_account
        .call(storage.id(), "new")
        .args_json(json!({"owner": user_account.id(), "collection": nft.id()}))
        .transact()
        .await?;

    assert!(initialize_storage.is_success());

    let owner = user_account
        .view(storage.id(), "owner")
        .args_json(json!({}))
        .await?
        .json::<AccountId>()?;

    let collection = user_account
        .view(storage.id(), "collection")
        .args_json(json!({}))
        .await?
        .json::<AccountId>()?;

    assert!(owner == *user_account.id());
    assert!(collection == *nft.id());

    Ok(())
}


#[tokio::test]
async fn contract_unlocks_token() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;

    let storage = near_workspaces::compile_project("./").await?;
    let nft = near_workspaces::compile_project("../nft/.").await?;
    let storage = sandbox.dev_deploy(&storage).await?;
    let nft = sandbox.dev_deploy(&nft).await?;

    let user_account = sandbox.dev_create_account().await?;

    let initialize_storage = user_account
        .call(storage.id(), "new")
        .args_json(json!({"owner": user_account.id(), "collection": nft.id()}))
        .transact()
        .await?;

    assert!(initialize_storage.is_success());

    let intialize_nft = user_account
        .call(nft.id(), "new_default_meta")
        .args_json(json!({"owner_id": user_account.id()}))
        .transact()
        .await?;
    assert!(intialize_nft.is_success());

    mint_nft(&user_account, &nft, "token-1").await?;

    transfer_nft(&user_account, &storage.as_account(), &nft, "token-1").await?;

    let unlock = user_account
        .call(storage.id(), "unlock_token")
        .args_json(json!({
            "to": user_account.id(),
            "token_id": "token-1"
        }))
        .transact()
        .await?;
    assert!(unlock.is_success());
    Ok(())
}
