## This repository contains the Aptos contract for the XP Bridge.

## Project structure

- `sources` - source code of all the moudules of the project.
- `tests` - tests for the contracts.

## How to use

### Install the CLI

https://aptos.dev/tools/aptos-cli/install-cli/

### Initalize New Local APTOS Account

`aptos init`

Press return to accept the default devnet network or specify the network of your choosing

See and respond to the prompt for your private key by accepting the default to create a new or by entering an existing key

This will output a account address and funds it with some Octas

### Build

`aptos move compile --dev` or `aptos move compile` for production

### Test

`npm test`

### Publish

`aptos move publish --dev` or `aptos move publish` for production
