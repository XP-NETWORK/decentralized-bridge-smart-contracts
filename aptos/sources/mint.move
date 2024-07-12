module bridge::mint {
  use aptos_token_objects::token::{Self};
  use std::string::{String};
  use std::option;
  use aptos_token_objects::collection::{Self, Collection};
  use aptos_framework::primary_fungible_store;
  use aptos_framework::fungible_asset::{Self};
  use std::signer;
  use aptos_framework::object;

  public entry fun mint_to(  
    creator: &signer,
    collection: String,
    collection_description: String,
    collection_uri: String,
    token_name: String,
    token_description: String,
    token_uri: String
  ) {
    let creator_addr = signer::address_of(creator);

    let collection_address = collection::create_collection_address(&creator_addr, &collection);
    let collection_exists = object::object_exists<Collection>(collection_address);
    if(!collection_exists) {
      collection::create_unlimited_collection(
          creator,
          collection_description,
          collection,
          option::none(),
          collection_uri,
      );
    };

    token::create_named_token(
        creator,
        collection,
        token_description,
        token_name,
        option::none(),
        token_uri,
    );
  }

  public entry fun mint_1155_to(  
    creator: &signer,
    collection: String,
    collection_description: String,
    collection_uri: String,
    token_name: String,
    token_description: String,
    token_uri: String,
    token_symbol: String,
    amount: u128,
    icon_uri: String,
    project_uri: String,
  ) {
    let creator_addr = signer::address_of(creator);

    let collection_address = collection::create_collection_address(&creator_addr, &collection);
    let collection_exists = object::object_exists<Collection>(collection_address);
    if(!collection_exists) {
      collection::create_unlimited_collection(
        creator,
        collection_description,
        collection,
        option::none(),
        collection_uri,
      );
    };

    let new_nft_constructor_ref = &token::create_named_token(
      creator,
      collection,
      token_description,
      token_name,
      option::none(),
      token_uri,
    );
    // Make this token fungible so there can multiple instances of it.
    primary_fungible_store::create_primary_store_enabled_fungible_asset(
      new_nft_constructor_ref,
      option::some(amount),
      token_name,
      token_symbol,
      0, // token cannot be divided so decimals is 0,
      icon_uri,
      project_uri,
    );


    let mint_ref = &fungible_asset::generate_mint_ref(new_nft_constructor_ref);
    let fa = fungible_asset::mint(mint_ref, (amount as u64));
    primary_fungible_store::deposit(creator_addr, fa);

  }
}