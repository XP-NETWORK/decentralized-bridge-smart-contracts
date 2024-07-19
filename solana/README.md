# XP.Network Solana Migration

## Installing solana
```
sh -c "$(curl -sSfL https://release.solana.com/v1.10.24/install)"
export PATH="~/.local/share/solana/install/active_release/bin:$PATH"
```
Reboot the terminal: close and open it again.

## Installing `anchor` on M1
```
cargo install --git https://github.com/project-serum/anchor --tag v0.24.2 anchor-cli --locked

anchor --version
```
Expected output:
```
anchor-cli 0.24.2
```

## Cloning the project
```
git clone https://github.com/XP-NETWORK/solana-bridge
cd solana-bridge
git submodule update --init
```

## Build Metaplex program to run on local

```
cd deps/metaplex/token-metadata
cargo build-bpf
```

## Building the program

```
cd ../../../
anchor build
```

## Testing the program locally

```
solana-keygen new
yarn add ts-mocha --dev
anchor test
```
All 8 tests pass.

## Deploying the program on the devnet

Run the airdrop command 4 times to get 8 SOL for deployment.

```
solana config set --url devnet
solana airdrop 2
solana airdrop 2
solana airdrop 2
solana airdrop 2
rm -rf target
anchor build
anchor deploy --program-keypair program-id.json --provider.cluster devnet --program-name xp_bridge
```
Program deployd on Devnet: https://explorer.solana.com/address/F3bscnmLEuFk2Fz9yxeY49TuxnGqLvsVKxmZgFjLFwGn?cluster=devnet

## Deploying the program on the testnet

Run the airdrop command 8 times to get 8 SOL for deployment.

```
solana config set --url testnet
solana airdrop 1
solana airdrop 1
solana airdrop 1
solana airdrop 1
solana airdrop 1
solana airdrop 1
solana airdrop 1
solana airdrop 1
anchor deploy --provider.cluster testnet
```

Program deployd on testnet: https://explorer.solana.com/address/F3bscnmLEuFk2Fz9yxeY49TuxnGqLvsVKxmZgFjLFwGn?cluster=testnet
