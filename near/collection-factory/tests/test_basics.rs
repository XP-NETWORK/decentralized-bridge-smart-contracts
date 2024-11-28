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
