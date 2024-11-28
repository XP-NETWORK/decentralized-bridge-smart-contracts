near delete-account xp-sf-test.testnet imsk17.testnet
near delete-account xp-cf-test.testnet imsk17.testnet
near delete-account xp-bridge-test.testnet imsk17.testnet

near create-account xp-cf-test.testnet --useFaucet
near create-account xp-sf-test.testnet --useFaucet
near create-account xp-bridge-test.testnet --useFaucet

near deploy xp-cf-test.testnet target/near/collection_factory/collection_factory.wasm --initFunction new --initArgs '{"owner": "xp-bridge-test.testnet"}'

near deploy xp-sf-test.testnet target/near/storage_factory/storage_factory.wasm --initFunction new --initArgs '{"owner": "xp-bridge-test.testnet"}'

near deploy xp-bridge-test.testnet target/near/bridge/bridge.wasm --initFunction new --initArgs '{"collection_factory": "xp-cf-test.testnet", "storage_factory": "xp-sf-test.testnet", "validators": [["xp-val1.testnet", "c426a2fb789a5c0541d2bbd26f8ccc9e4305b477f188a08024fcb53f20ef135b"], ["xp-val2.testnet","a19641af6cadb775bba703c81d629edaeea5fb5b0526c32b342a0b778b103504"]]}'
