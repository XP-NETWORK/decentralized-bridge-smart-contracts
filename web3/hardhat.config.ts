import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";
import "@openzeppelin/hardhat-upgrades";

const config: HardhatUserConfig = {
    solidity: {
        version: "0.8.21",
        settings: {
            outputSelection: {
                "*": {
                    "*": ["*"],
                },
            },
            viaIR: true,
            optimizer: {
                enabled: true,
                runs: 50,
            },
        },
    },
    networks: {
        hardhat: {}, // Local Ethereum network 
        bscTestnet: {
            url: "https://data-seed-prebsc-1-s1.binance.org:8545/", // BSC Testnet RPC URL
            chainId: 97, // Chain ID for BSC Testnet
            gasPrice: 5000000000, // Gas price (wei) for transactions on BSC Testnet
            accounts: [
                "8a6a4e9e3865814dc8f1e4fca103051162921715d1e517ba638ba05656f320b0",
            ],
        },
        ethTestnet: {
            url: "https://rpc.notadegen.com/eth/sepolia", // ETH Testnet RPC URL
            chainId: 11155111, // Chain ID for BSC Testnet
            gasPrice: 9000000000, // Gas price (wei) for transactions on BSC Testnet
            accounts: [
                "8a6a4e9e3865814dc8f1e4fca103051162921715d1e517ba638ba05656f320b0",
            ],
        },
        opTestnet: {
            url: "https://optimism-goerli.publicnode.com", // OP Testnet RPC URL
            chainId: 420, // Chain ID for OP Testnet
            gasPrice: 5000000000, // Gas price (wei) for transactions on OP Testnet
            accounts: [
                "8a6a4e9e3865814dc8f1e4fca103051162921715d1e517ba638ba05656f320b0",
            ],
        }, 
        maticTestnet: {
            url: "https://rpc-mumbai.maticvigil.com/", // MATIC Testnet RPC URL
            chainId: 80001, // Chain ID for MATIC Testnet
            gasPrice: 5000000000, // Gas price (wei) for transactions on MATIC Testnet
            accounts: [
                "8a6a4e9e3865814dc8f1e4fca103051162921715d1e517ba638ba05656f320b0",
            ],
        },
    },
    typechain: {
        outDir: "contractsTypes",
    },
};

export default config;
