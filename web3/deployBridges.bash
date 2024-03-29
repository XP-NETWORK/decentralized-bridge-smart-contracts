# Array of chain symbols
declare -a chains=("BSC" "ETH" "MATIC")

# Bootstrap validator address
BOOTSTRAP_VALIDATOR_ADDRESS="0xdca3EB00DfaDeD529691736c4c7Ee386BFAE7c23"

# Loop over the chain symbols and execute the deployBridge task
for chain in "${chains[@]}"
do
  echo "Deploying Bridge for $chain"
  npx hardhat deployBridge --bootstrap-validator-address $BOOTSTRAP_VALIDATOR_ADDRESS --chain-symbol $chain --network ${chain,,}Testnet
done

