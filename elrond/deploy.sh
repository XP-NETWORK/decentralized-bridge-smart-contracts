WASM_PATH="./output/bridge.wasm"
WALLET_PEM="./test_wallet.json"
PWD = oy00ngy9xf


deploySC() {
    mxpy --verbose contract deploy --recall-nonce \
        --bytecode=./output/bridge.wasm \
        --keyfile=./test_wallet.json \
        --gas-limit=500000000 \
        --proxy https://devnet-gateway.multiversx.com \
        --arguments 0x7e1550d0230d9010ad618d36488993064b9694daba132567a66567aa7212c966 \
        --send || return
}
