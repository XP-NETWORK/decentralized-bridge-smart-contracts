import { InMemorySigner } from "@taquito/signer";
import { MichelsonMap, TezosToolkit } from "@taquito/taquito";
import { config } from "dotenv";
import { createInterface } from "readline/promises";
import BigNumber from "bignumber.js";

const stdio = createInterface({
  input: process.stdin,
  output: process.stdout,
});

config();

import Bridge from "../artifacts/Bridge.json";
import { deployStorageFactory } from "./deploy-storage-factory";
import { deployCollectionFactory } from "./deploy-collection-factory";

const RPC_ENDPOINT = process.env.RPC_ENDPOINT!;
const SK = process.env.SK!;

export async function deployBridge(validators_str: string[]) {
  const Tezos = new TezosToolkit(RPC_ENDPOINT);

  Tezos.setProvider({
    signer: await InMemorySigner.fromSecretKey(SK),
  });

  const validators = new MichelsonMap();
  validators_str.forEach((validator) => {
    validators.set(validator, new BigNumber(0));
  });

  const storage_deployer = await deployStorageFactory(undefined);
  if (!storage_deployer) {
    console.log("Error deploying storage factory");
    return;
  }
  const collection_deployer = await deployCollectionFactory(undefined);
  if (!collection_deployer) {
    console.log("Error deploying collection factory");
    return;
  }

  try {
    const originated = await Tezos.contract.originate({
      code: Bridge,
      storage: {
        validators,
        storage_deployer,
        collection_deployer,
        unique_identifiers: new MichelsonMap(),
        original_to_duplicate_mapping: new MichelsonMap(),
        duplicate_to_original_mapping: new MichelsonMap(),
        original_storage_mapping_nft: new MichelsonMap(),
        original_storage_mapping_sft: new MichelsonMap(),
        duplicate_storage_mapping_nft: new MichelsonMap(),
        duplicate_storage_mapping_sft: new MichelsonMap(),
        validators_count: validators.size,
      },
    });
    console.log("Originated: ", originated.contractAddress);
    const std = await Tezos.contract.at(storage_deployer);

    const owtf1 = await std.methods
      .set_owner(originated.contractAddress)
      .send();
    console.log(
      `Transferred ownership of Storage Deployer ${storage_deployer} to Bridge ${owtf1.hash}`
    );
    const ctd = await Tezos.contract.at(collection_deployer);
    const owtf2 = await ctd.methods
      .set_owner(originated.contractAddress)
      .send();
    console.log(
      `Transferred ownership of Collection Deployer ${collection_deployer} to Bridge ${owtf2.hash}`
    );

    await originated.confirmation();
    console.log("Bridge Contract: ", originated.contractAddress);
    return originated.contractAddress;
  } catch (error: any) {
    console.log(error);
    return undefined;
  }
}

function raise(message: string): never {
  throw new Error(message);
}

if (require.main === module) {
  (async () => {
    const ownerStr: string = await stdio.question(
      "Enter validator addresses (seprated by `,`): "
    );
    const validators =
      ownerStr !== "" ? ownerStr : raise("Validators are required");

    const validators_arr = validators.split(",");

    await deployBridge(validators_arr);
  })();
}
