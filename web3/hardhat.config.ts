import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";
import "@openzeppelin/hardhat-upgrades";
import "./prodScripts/deployBridge";

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
        runs: 200,
      },
    },
  },
  networks: {
    testnet: {
      url: "http://127.0.0.1:8545", // BSC Testnet RPC URL
      chainId: 31337, // Chain ID for BSC Testnet
      gasPrice: 5000000000, // Gas price (wei) for transactions on BSC Testnet
      accounts: [
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
      ],
    },
    hardhat: {}, // Local Ethereum network
    bscTestnet: {
      url: "https://bsc-testnet.publicnode.com", // BSC Testnet RPC URL
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
        "0x0e979ae1299df55645e68808754c93c067e35834195c420945d062858bea2965",
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
    hederaTestnet: {
      url: "https://testnet.hashio.io/api",
      accounts: [
        "0cfdd2caea2ca7ae542829e984272f3c07ee18e251fe35f8d99ee8270d3abdb5",
      ],
      timeout: 50000000000
    },
    hederaLocal: {
      url: "http://localhost:7546/",
      accounts: [
        "0cfdd2caea2ca7ae542829e984272f3c07ee18e251fe35f8d99ee8270d3abdb5",
      ],
      gas: 5000000,
      timeout: 50000000000
    },
    bscMainnet: {
      url: "https://bsc-pokt.nodies.app", // BSC Mainnet RPC URL
      accounts: [
        "bdc6d05d061c7bb43fb931b9c4e9b04e0b15ecdcf513f9787b2143f596da0cf0",
      ],
    },
    ethMainnet: {
      url: "https://eth.llamarpc.com", // ETH Mainnet RPC URL
      accounts: [
        "bdc6d05d061c7bb43fb931b9c4e9b04e0b15ecdcf513f9787b2143f596da0cf0",
      ],
    },
    maticMainnet: {
      url: "https://polygon-pokt.nodies.app", // Polygon Mainnet RPC URL
      accounts: [
        "bdc6d05d061c7bb43fb931b9c4e9b04e0b15ecdcf513f9787b2143f596da0cf0",
      ],
    },
    hederaMainnet: {
      url: "https://mainnet.hashio.io/api", // Hedera Mainnet RPC URL
      accounts: [
        "bdc6d05d061c7bb43fb931b9c4e9b04e0b15ecdcf513f9787b2143f596da0cf0",
      ],
    },
    optMainnet: {
      url: "https://optimism-mainnet.public.blastapi.io", // Optimism Mainnet RPC URL
      accounts: [
        "bdc6d05d061c7bb43fb931b9c4e9b04e0b15ecdcf513f9787b2143f596da0cf0",
      ],
    },
  },
  typechain: {
    outDir: "contractsTypes",
  },
};

export default config;