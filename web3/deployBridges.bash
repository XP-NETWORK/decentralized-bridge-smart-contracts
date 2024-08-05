# Array of chain symbols
declare -a chains=("BSC" "MATIC")

# Bootstrap validator address
BOOTSTRAP_VALIDATOR_ADDRESS="0xe7D463DFf4E8c01040DafD137598d006292A7Aa3"

# Loop over the chain symbols and execute the deployBridge task
for chain in "${chains[@]}"
do
  echo "Deploying Bridge for $chain"
  npx hardhat deployBridge --bootstrap-validator-address $BOOTSTRAP_VALIDATOR_ADDRESS --chain-symbol $chain --network ${chain,,}Mainnet
done

