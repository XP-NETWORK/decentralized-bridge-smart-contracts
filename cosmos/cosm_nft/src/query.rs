use crate::msg::{CheckRoyaltiesResponse, RoyaltiesInfoResponse};
use crate::NftContract;
use cosmwasm_std::{Decimal, Deps, StdResult, Uint128};

/// NOTE: default behaviour here is to round down
/// EIP2981 specifies that the rounding behaviour is at the discretion of the implementer
pub fn query_royalties_info(
    deps: Deps,
    token_id: String,
    sale_price: Uint128,
) -> StdResult<RoyaltiesInfoResponse> {
    let contract = NftContract::default();
    let token_info = contract.tokens.load(deps.storage, &token_id)?;

    let royalty_percentage = Decimal::percent(token_info.extension.royalty_percentage);
    let royalty_from_sale_price = sale_price * royalty_percentage;

    let royalty_address = token_info.extension.royalty_payment_address;

    Ok(RoyaltiesInfoResponse {
        address: royalty_address,
        royalty_amount: royalty_from_sale_price,
    })
}

/// As our default implementation here specifies royalties at token level
/// and not at contract level, it is therefore logically true that
/// on sale, every token managed by this contract should be checked
/// to see if royalties are owed, and to whom. If you are importing
/// this logic, you may want a custom implementation here
pub fn check_royalties(_deps: Deps) -> StdResult<CheckRoyaltiesResponse> {
    Ok(CheckRoyaltiesResponse {
        royalty_payments: true,
    })
}
