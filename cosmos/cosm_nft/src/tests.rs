
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::contract::methods;
//     use crate::init::InstantiateMsg;
//     use crate::msg::{CheckRoyaltiesResponse, RoyaltiesInfoResponse};
//     use crate::{NftContract, NftExecuteMsg};

//     use cosmwasm_std::{from_json, Uint128};

//     use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
//     use cw721_base::QueryMsg;

//     const CREATOR: &str = "creator";

//     #[test]
//     fn use_metadata_extension() {
//         let mut deps = mock_dependencies();
//         let contract = NftContract::default();

//         let info = mock_info(CREATOR, &[]);
//         let init_msg = InstantiateMsg {
//             name: "SpaceShips".to_string(),
//             symbol: "SPACE".to_string(),
//             minter: "Addr".to_string()
//         };
//         methods::init(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

//         let token_id = "Enterprise";
//         let token_uri = Some("https://starships.example.com/Starship/Enterprise.json".into());
//         let extension = crate::RoyaltyData{ royalty_percentage: todo!(), royalty_payment_address: todo!() };
//         let exec_msg = NftExecuteMsg::Mint {
//             token_id: token_id.to_string(),
//             owner: "john".to_string(),
//             token_uri: token_uri.clone(),
//             extension
//         };
//         methods::execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

//         let res = contract.query(deps.as_ref(), mock_env(), QueryMsg::Extension { msg: CheckRoyaltiesResponse  }).unwrap();
//         assert_eq!(res, token_uri);
//         assert_eq!(res.extension, extension);
//     }

//     #[test]
//     fn validate_royalty_information() {
//         let mut deps = mock_dependencies();
//         let _contract = Cw2981Contract::default();

//         let info = mock_info(CREATOR, &[]);
//         let init_msg = InstantiateMsg {
//             name: "SpaceShips".to_string(),
//             symbol: "SPACE".to_string(),
//             minter: None,
//             withdraw_address: None,
//         };
//         entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

//         let token_id = "Enterprise";
//         let exec_msg = ExecuteMsg::Mint {
//             token_id: token_id.to_string(),
//             owner: "john".to_string(),
//             token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
//             extension: Some(Metadata {
//                 description: Some("Spaceship with Warp Drive".into()),
//                 name: Some("Starship USS Enterprise".to_string()),
//                 royalty_percentage: Some(101),
//                 ..Metadata::default()
//             }),
//         };
//         // mint will return StdError
//         let err = entry::execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap_err();
//         assert_eq!(err, ContractError::InvalidRoyaltyPercentage);
//     }

//     #[test]
//     fn check_royalties_response() {
//         let mut deps = mock_dependencies();
//         let _contract = Cw2981Contract::default();

//         let info = mock_info(CREATOR, &[]);
//         let init_msg = InstantiateMsg {
//             name: "SpaceShips".to_string(),
//             symbol: "SPACE".to_string(),
//             minter: None,
//             withdraw_address: None,
//         };
//         entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

//         let token_id = "Enterprise";
//         let exec_msg = ExecuteMsg::Mint {
//             token_id: token_id.to_string(),
//             owner: "john".to_string(),
//             token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
//             extension: Some(Metadata {
//                 description: Some("Spaceship with Warp Drive".into()),
//                 name: Some("Starship USS Enterprise".to_string()),
//                 ..Metadata::default()
//             }),
//         };
//         entry::execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

//         let expected = CheckRoyaltiesResponse {
//             royalty_payments: true,
//         };
//         let res = check_royalties(deps.as_ref()).unwrap();
//         assert_eq!(res, expected);

//         // also check the longhand way
//         let query_msg = QueryMsg::Extension {
//             msg: Cw2981QueryMsg::CheckRoyalties {},
//         };
//         let query_res: CheckRoyaltiesResponse =
//             from_json(entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
//         assert_eq!(query_res, expected);
//     }

//     #[test]
//     fn check_token_royalties() {
//         let mut deps = mock_dependencies();

//         let info = mock_info(CREATOR, &[]);
//         let init_msg = InstantiateMsg {
//             name: "SpaceShips".to_string(),
//             symbol: "SPACE".to_string(),
//             minter: None,
//             withdraw_address: None,
//         };
//         entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

//         let token_id = "Enterprise";
//         let owner = "jeanluc";
//         let exec_msg = ExecuteMsg::Mint {
//             token_id: token_id.to_string(),
//             owner: owner.into(),
//             token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
//             extension: Some(Metadata {
//                 description: Some("Spaceship with Warp Drive".into()),
//                 name: Some("Starship USS Enterprise".to_string()),
//                 royalty_payment_address: Some("jeanluc".to_string()),
//                 royalty_percentage: Some(10),
//                 ..Metadata::default()
//             }),
//         };
//         entry::execute(deps.as_mut(), mock_env(), info.clone(), exec_msg).unwrap();

//         let expected = RoyaltiesInfoResponse {
//             address: owner.into(),
//             royalty_amount: Uint128::new(10),
//         };
//         let res =
//             query_royalties_info(deps.as_ref(), token_id.to_string(), Uint128::new(100)).unwrap();
//         assert_eq!(res, expected);

//         // also check the longhand way
//         let query_msg = QueryMsg::Extension {
//             msg: Cw2981QueryMsg::RoyaltyInfo {
//                 token_id: token_id.to_string(),
//                 sale_price: Uint128::new(100),
//             },
//         };
//         let query_res: RoyaltiesInfoResponse =
//             from_json(entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
//         assert_eq!(query_res, expected);

//         // check for rounding down
//         // which is the default behaviour
//         let voyager_token_id = "Voyager";
//         let owner = "janeway";
//         let voyager_exec_msg = ExecuteMsg::Mint {
//             token_id: voyager_token_id.to_string(),
//             owner: owner.into(),
//             token_uri: Some("https://starships.example.com/Starship/Voyager.json".into()),
//             extension: Some(Metadata {
//                 description: Some("Spaceship with Warp Drive".into()),
//                 name: Some("Starship USS Voyager".to_string()),
//                 royalty_payment_address: Some("janeway".to_string()),
//                 royalty_percentage: Some(4),
//                 ..Metadata::default()
//             }),
//         };
//         entry::execute(deps.as_mut(), mock_env(), info, voyager_exec_msg).unwrap();

//         // 43 x 0.04 (i.e., 4%) should be 1.72
//         // we expect this to be rounded down to 1
//         let voyager_expected = RoyaltiesInfoResponse {
//             address: owner.into(),
//             royalty_amount: Uint128::new(1),
//         };

//         let res = query_royalties_info(
//             deps.as_ref(),
//             voyager_token_id.to_string(),
//             Uint128::new(43),
//         )
//         .unwrap();
//         assert_eq!(res, voyager_expected);
//     }
// }