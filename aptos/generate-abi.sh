#! /bin/bash
 
# replace it with the network your contract lives on
NETWORK=testnet
# replace it with your contract address
CONTRACT_ADDRESS=13f8d626e383e8621a89caeb05c56a95fda38aa2dddfa8c2b1ed063f0edb23c9
# replace it with your module name, every .move file except move script has module_address::module_name {}
MODULE_NAME=aptos_nft_bridge
 
# save the ABI to a TypeScript file
echo "export const ABI = $(curl https://fullnode.$NETWORK.aptoslabs.com/v1/accounts/$CONTRACT_ADDRESS/module/$MODULE_NAME | sed -n 's/.*"abi":\({.*}\).*}$/\1/p') as const" > abi.ts