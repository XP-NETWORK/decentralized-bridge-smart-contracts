import { SecretNetworkClient, Wallet } from "secretjs";
import { readFileSync, writeFileSync } from "fs";

export async function deploy() {
  const wallet = new Wallet(process.env.WALLET!);

  const sc = new SecretNetworkClient({
    chainId: "pulsar-3",
    url: "https://api.pulsar.scrttestnet.com",
    wallet,
    walletAddress: wallet.address,
  });

  const accounts = await wallet.getAccounts();
  const validator = (await wallet.getAccounts())[0];
  console.log("Validator:", validator.address);

  console.log(
    `Balance:`,
    await sc.query.bank.balance({
      address: validator.address,
      denom: "uscrt",
    })
  );

  const snip721_wasm = readFileSync("./artifacts/snip721.wasm.gz");
  const snip1155_wasm = readFileSync("./artifacts/snip1155.wasm.gz");
  const collection_deployer_wasm = readFileSync(
    "./artifacts/collection_deployer.wasm.gz"
  );
  const storage721_wasm = readFileSync("./artifacts/storage721.wasm.gz");
  const storage1155_wasm = readFileSync("./artifacts/storage1155.wasm.gz");
  const storage_deployer_wasm = readFileSync("./artifacts/storage_deployer.wasm.gz");
  const bridge_wasm = readFileSync("./artifacts/bridge.wasm.gz");

  const uploadNft = await sc.tx.compute.storeCode(
    {
      wasm_byte_code: snip721_wasm,
      sender: validator.address,
      source: "",
      builder: "",
    },
    {
      gasLimit: 5000000,
    }
  );
  console.log(uploadNft)
  const nftCodeId = uploadNft.arrayLog?.find((e) => e.key === "code_id")?.value!;
  console.log("Uploaded Nft contract code. Code ID: ", nftCodeId);

  const uploadSft = await sc.tx.compute.storeCode(
    {
      wasm_byte_code: snip1155_wasm,
      sender: validator.address,
      source: "",
      builder: "",
    },
    {
      gasLimit: 5000000,
    }
  );
  const sftCodeId = uploadSft.arrayLog?.find((e) => e.key === "code_id")?.value!;
  console.log("Uploaded SFT contract code. Code ID: ", sftCodeId);

  const uploadCd = await sc.tx.compute.storeCode(
    {
      wasm_byte_code: collection_deployer_wasm,
      sender: validator.address,
      source: "",
      builder: "",
    },
    {
      gasLimit: 5000000,
    }
  );
  const cdCodeId = uploadCd.arrayLog?.find((e) => e.key === "code_id")?.value!;
  console.log("Uploaded CD contract code. Code ID: ", cdCodeId);

  const uploadStorage721 = await sc.tx.compute.storeCode(
    {
      wasm_byte_code: storage721_wasm,
      sender: validator.address,
      source: "",
      builder: "",
    },
    {
      gasLimit: 5000000,
    }
  );
  const storage721CodeId = uploadStorage721.arrayLog?.find(
    (e) => e.key === "code_id"
  )?.value!;
  console.log("Uploaded Storage721 contract code. Code ID: ", storage721CodeId);

  const uploadStorage1155 = await sc.tx.compute.storeCode(
    {
      wasm_byte_code: storage1155_wasm,
      sender: validator.address,
      source: "",
      builder: "",
    },
    {
      gasLimit: 5000000,
    }
  );
  const storage1155CodeId = uploadStorage1155.arrayLog?.find(
    (e) => e.key === "code_id"
  )?.value!;
  console.log(
    "Uploaded Storage1155 contract code. Code ID: ",
    storage1155CodeId
  );

  const uploadStorageDeployer = await sc.tx.compute.storeCode(
    {
      wasm_byte_code: storage_deployer_wasm,
      sender: validator.address,
      source: "",
      builder: "",
    },
    {
      gasLimit: 5000000,
    }
  );
  const storageDeployerCodeId = uploadStorageDeployer.arrayLog?.find(
    (e) => e.key === "code_id"
  )?.value!;

  const uploadBridge = await sc.tx.compute.storeCode(
    {
      wasm_byte_code: bridge_wasm,
      sender: validator.address,
      source: "",
      builder: "",
    },
    {
      gasLimit: 5000000,
    }
  );
  const bridgeCodeId = uploadBridge.arrayLog?.find(
    (e) => e.key === "code_id"
  )?.value!;
  console.log("Uploaded Bridge contract code. Code ID: ", bridgeCodeId);

  const initMsg = {
    validators: [
      [Buffer.from(validator.pubkey).toString("base64"), validator.address],
    ],
    chain_type: "SECRET",
    storage_label: "XP NFT Store Deployer",
    collection_label: "XP NFT Collection Deployer",
    collection721_code_info: {
      code_id: parseInt(nftCodeId),
      code_hash: (
        await sc.query.compute.codeHashByCodeId({
          code_id: nftCodeId,
        })
      ).code_hash,
    },
    storage721_code_info: {
      code_id: parseInt(storage721CodeId),
      code_hash: (
        await sc.query.compute.codeHashByCodeId({
          code_id: storage721CodeId,
        })
      ).code_hash,
    },
    collection1155_code_info: {
      code_id: parseInt(sftCodeId),
      code_hash: (
        await sc.query.compute.codeHashByCodeId({
          code_id: sftCodeId,
        })
      ).code_hash,
    },
    storage1155_code_info: {
      code_id: parseInt(storage1155CodeId),
      code_hash: (
        await sc.query.compute.codeHashByCodeId({
          code_id: storage1155CodeId,
        })
      ).code_hash,
    },
    collection_deployer_code_info: {
      code_id: parseInt(cdCodeId),
      code_hash: (
        await sc.query.compute.codeHashByCodeId({
          code_id: cdCodeId,
        })
      ).code_hash,
    },
    storage_deployer_code_info: {
      code_id: parseInt(storageDeployerCodeId),
      code_hash: (
        await sc.query.compute.codeHashByCodeId({
          code_id: storageDeployerCodeId,
        })
      ).code_hash,
    },
  };
  console.log(initMsg)

    const exec = await sc.tx.compute.instantiateContract(
      {
        code_id: bridgeCodeId,
        init_msg: initMsg,
        label: "XP Bridge V3",
        sender: validator.address,
      },

      {
        gasLimit: 5000000,
      }
    );
    writeFileSync("./code_id.json", JSON.stringify({
      nftCodeId,
      storage721CodeId,
      storage1155CodeId,
      storageDeployerCodeId,
      cdCodeId,
      sftCodeId,
      bridgeCodeId
    }))
    console.log(`Bridge instantiate result:`, JSON.stringify(exec));
}

deploy();
