
echo $1
near create-account xp-cf-main.near --accountId $1 5 # 5 NEAR SHOULD BE GIVEN
near create-account xp-sf-main.near --accountId $1 5 # 5 NEAR SHOULD BE GIVEN
near create-account xp-bridge-main.near --accountId $1 5 # 5 NEAR SHOULD BE GIVEN

near deploy xp-cf-main.near target/near/collection_factory/collection_factory.wasm --initFunction new --initArgs '{"owner": "xp-bridge-main.near"}'

near deploy xp-sf-test.near target/near/storage_factory/storage_factory.wasm --initFunction new --initArgs '{"owner": "xp-bridge-main.near"}'

near deploy xp-bridge-main.near target/near/bridge/bridge.wasm --initFunction new --initArgs '{"collection_factory": "xp-cf-main.near", "storage_factory": "xp-sf-main.near", "validators": [["<PUBLICKEYINHEX>", "xp-val1.near"], ["<PUBLICKEYINHEX>", "xp-val2.near"]]}'


