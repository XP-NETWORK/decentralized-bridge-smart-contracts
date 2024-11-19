// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           13
// Async Callback:                       1
// Total number of exported functions:  15

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    bridge
    (
        init => init
        tokens => tokens
        validators => validators
        validatorsCount => validators_count
        uniqueIdentifier => unique_identifier
        originalToDuplicateMapping => original_to_duplicate_mapping
        duplicateToOriginalMapping => duplicate_to_original_mapping
        addValidator => add_validator
        claimValidatorRewards => claim_validator_rewards
        lock721 => lock721
        lock1155 => lock1155
        claimNft721 => claim_nft721
        claimNft1155 => claim_nft1155
        collections => collections
    )
}

multiversx_sc_wasm_adapter::async_callback! { bridge }
