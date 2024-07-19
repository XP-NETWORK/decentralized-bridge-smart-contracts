## This repository contains the Aptos contract for the XP Bridge.

## Project structure

- `sources` - source code of all the moudules of the project.
- `tests` - unit tests for the contracts.
- `src` - ts code for bridge client

## How to use

### Install the CLI

https://aptos.dev/tools/aptos-cli/install-cli/

### Initalize New Local APTOS Account

`cd aptos`

`aptos init`

Select network

This will output a account address and funds it with some Octas

Copy the private key from the generated .aptos/config.yaml file and put inside .env

Copy the account address and put inside Move.toml and src/constants

Similarly create accounts for validators, nft owner and test account somewhere else and paste thier private keys inside .env

### Build

`aptos move compile --dev` or `aptos move compile` for production

### Test

`npm test`

### Publish

`aptos move publish --dev` or `aptos move publish` for production
