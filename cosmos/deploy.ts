import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing";
import {readFile} from "fs/promises";

const storage = await Bun.file("storage.json", ).json()

const directWallet = await DirectSecp256k1Wallet.fromKey(
  Buffer.from(process.env.SK!, "hex"),
  process.env.PREFIX
);
const client = await SigningCosmWasmClient.connectWithSigner(
  process.env.URL!,
  directWallet
);

const v1 = await DirectSecp256k1Wallet.fromKey(
  Buffer.from(process.env.V1!, "hex"),
  process.env.PREFIX
);
const v2 = await DirectSecp256k1Wallet.fromKey(
  Buffer.from(process.env.V2!, "hex"),
  process.env.PREFIX
);

const signer = (await directWallet.getAccounts())[0];

const contracts = {
  Nft: "cosm_nft.wasm",
  NftStore: "nft_store.wasm",
  StoreDeployer: "store_deployer.wasm",
  CollectionDeployer: "collection_deployer.wasm",
  Bridge: "bridge.wasm",
};

for (const contract of Object.values(contracts)) {
    if (storage[contract] !== undefined) {
        console.log(`Skipping ${contract} as it is already deployed`);
        continue;
    }
    const upload = await client.upload(
      signer.address,
      await readFile(`artifacts/${contract}`),
      {
        amount: [{ amount: "75000", denom: "uluna" }],
        gas: "5000000",
      }
    );
    storage[contract] = upload.codeId;
    Bun.write(Bun.file("storage.json"), JSON.stringify(storage));
    console.log("Upload succeeded. Receipt: ", upload);
}

const instBridge = async (codeId: number) => {
  const storageCodeId = storage[contracts.NftStore];
  const collectionCodeId = storage[contracts.Nft];
  const cdCodeId = storage[contracts.CollectionDeployer];
  const sdCodeId = storage[contracts.StoreDeployer];
  const init = await client.instantiate(
    signer.address,
    codeId,
    {
      validators: [
        [
          Buffer.from((await v1.getAccounts())[0].pubkey).toString("base64"),
          (await v1.getAccounts())[0].address,
        ],
        [
          Buffer.from((await v2.getAccounts())[0].pubkey).toString("base64"),
          (await v2.getAccounts())[0].address,
        ],
      ],
      chain_type: "TERRA",
      storage_label: "xp-storage-1",
      collection_label: "xp-collection-1",
      collection721_code_id: collectionCodeId,
      storage721_code_id: storageCodeId,
      collection_deployer_code_id: cdCodeId,
      storage_deployer_code_id: sdCodeId,
    },
    `xp-bridge-${codeId}`,
    {
      amount: [{ amount: "75000", denom: "uluna" }],
      gas: "5000000",
    }
  );
  return init;
}
const init = await instBridge(storage[contracts.Bridge]);
console.log("Bridge init succeeded. Receipt: ", init);