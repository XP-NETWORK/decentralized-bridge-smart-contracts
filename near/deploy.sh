near delete-account xp-sf-test.testnet imsk17.testnet
near delete-account xp-cf-test.testnet imsk17.testnet
near delete-account xp-bridge-test.testnet imsk17.testnet

near create-account xp-cf-test.testnet --useFaucet
near create-account xp-sf-test.testnet --useFaucet
near create-account xp-bridge-test.testnet --useFaucet

near deploy xp-cf-test.testnet target/near/collection_factory/collection_factory.wasm --initFunction new --initArgs '{"owner": "xp-bridge-test.testnet"}'

near deploy xp-sf-test.testnet target/near/storage_factory/storage_factory.wasm --initFunction new --initArgs '{"owner": "xp-bridge-test.testnet"}'

near deploy xp-bridge-test.testnet target/near/bridge/bridge.wasm --initFunction new --initArgs '{"collection_factory": "xp-cf-test.testnet", "storage_factory": "xp-sf-test.testnet", "validators": [["xp-val1.testnet", "37a03f86504c3054b375f57066051b75e1fcec1e69413f98140ca9d4996a11fb"]]}'
