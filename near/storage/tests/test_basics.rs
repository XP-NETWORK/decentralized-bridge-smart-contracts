use serde_json::json;

#[tokio::test]
async fn test_contract_is_operational() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let nft = sandbox
        .dev_deploy(include_bytes!("../../target/near/nft/nft.wasm"))
        .await?;
    // let nft = sandbox.dev_deploy(&nft_wasm).await?;

    let user_account = sandbox.dev_create_account().await?;

    let intialize_nft = user_account
        .call(nft.id(), "new_default_meta")
        .args_json(json!({"owner_id": user_account.id()}))
        .transact()
        .await?;
    assert!(intialize_nft.is_success());

    let mint = user_account
        .call(nft.id(), "nft_mint")
        .args_json(json!({
            "token_id": "1",
            "metadata": { "title": "Minted Token", "description": "This token was minted by the contract" },
            "receiver_id": user_account.id().to_string(),
            "perpetual_royalties": { "user_account.near": 100 }
        }))
        .transact()
        .await?;
    eprintln!("{:#?}", mint);
    assert!(mint.is_success());
    Ok(())
}
