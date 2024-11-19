WASM_PATH="./output/bridge.wasm"
WALLET_PEM="./test_wallet.json"
PWD = oy00ngy9xf


deploySC() {
    mxpy --verbose contract deploy --recall-nonce \
        --bytecode=./output/bridge.wasm \
        --keyfile=./test_wallet.json \
        --gas-limit=500000000 \
        --proxy https://devnet-gateway.multiversx.com \
        --arguments 0x9fb927c978225cb7a93b8b3cd8d8423e176e009dc284c536d9c4372bbe128487 \
        --send || return
}
