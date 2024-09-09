#![allow(unused)]
use serde_json::json;
use near_workspaces::{types::{NearToken, AccountDetails}, Account, Contract};

pub const DEFAULT_DEPOSIT: u128 = 10000000000000000000000;
pub const ONE_YOCTO_NEAR: NearToken = NearToken::from_yoctonear(1);

pub async fn mint_nft(
    user: &Account,
    nft_contract: &Contract,
    token_id: &str,
) -> Result<(), Box<dyn std::error::Error>> { 
    let request_payload = json!({
        "token_id": token_id,
        "receiver_id": user.id(),
        "metadata": {
            "title": "Grumpy Cat",
            "description": "Not amused.",
            "media": "https://www.adamsdrafting.com/wp-content/uploads/2018/06/More-Grumpy-Cat.jpg"
        },
    });

    let mint = user.call(nft_contract.id(), "nft_mint")
        .args_json(request_payload)
        .deposit(NearToken::from_yoctonear(DEFAULT_DEPOSIT))
        .transact()
        .await?;
    assert!(mint.is_success());
    Ok(())
}

pub async fn approve_nft(
    market_contract: &Contract,
    user: &Account,
    nft_contract: &Contract,
    token_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_payload  = json!({
        "token_id": token_id,
        "account_id": market_contract.id(),
        "msg": serde_json::Value::Null,
    });

    let _ = user.call(nft_contract.id(), "nft_approve")
        .args_json(request_payload)
        .deposit(NearToken::from_yoctonear(DEFAULT_DEPOSIT))
        .transact()
        .await;

    Ok(())
}
