import {
  Aptos,
  Ed25519Account,
  PendingTransactionResponse,
  Serializer,
} from "@aptos-labs/ts-sdk";
import { BRIDGE_ADDRESS, BRIDGE_MODULE, BRIDGE_FUNCTIONS } from "./constants";
import { createHash } from "crypto";
import * as ed from "@noble/ed25519";

type TCollectionCounterObj = {
  key: string;
  value: string;
};

type TValidatorsObj = {
  key: string;
  value: {
    pending_reward: string;
  };
};

type TBridgeData = {
  collection_objects: {
    handle: string;
  };
  duplicate_to_original_mapping: {
    handle: string;
  };
  nft_collection_tokens: {
    handle: string;
  };
  nft_collections_counter: {
    data: TCollectionCounterObj[];
  };
  nfts_counter: string;
  original_to_duplicate_mapping: {
    handle: string;
  };
  self_chain: string;
  validators: {
    data: TValidatorsObj[];
  };
  signer_cap: {
    account: string;
  };
};

export class BridgeClient {
  private aptosClient: Aptos;

  constructor(client: Aptos) {
    this.aptosClient = client;
  }

  async fundAccounts(accounts: Ed25519Account[]): Promise<void> {
    await Promise.all(
      accounts.map((account) =>
        this.aptosClient.fundAccount({
          accountAddress: account.accountAddress,
          amount: 100,
          options: { checkSuccess: true },
        })
      )
    );
  }

  async initialize(
    adminAccount: Ed25519Account,
    validators: Uint8Array[],
    seed: Uint8Array,
    selfChain: Uint8Array
  ): Promise<PendingTransactionResponse> {
    try {
      const transaction = await this.aptosClient.transaction.build.simple({
        sender: adminAccount.accountAddress,
        data: {
          function: `${BRIDGE_ADDRESS}::${BRIDGE_MODULE}::${BRIDGE_FUNCTIONS.Initialize}`,
          functionArguments: [validators, seed, selfChain],
        },
      });

      return this.aptosClient.signAndSubmitTransaction({
        signer: adminAccount,
        transaction,
      });
    } catch (error) {
      throw error;
    }
  }

  async addValidator(
    adminAccount: Ed25519Account,
    validator: Uint8Array,
    signatures: Uint8Array[],
    public_keys: Uint8Array[]
  ) {
    try {
      const transaction = await this.aptosClient.transaction.build.simple({
        sender: adminAccount.accountAddress,
        data: {
          function: `${BRIDGE_ADDRESS}::${BRIDGE_MODULE}::${BRIDGE_FUNCTIONS.AddValidator}`,
          functionArguments: [validator, signatures, public_keys],
        },
      });

      return this.aptosClient.signAndSubmitTransaction({
        signer: adminAccount,
        transaction,
      });
    } catch (error) {
      throw error;
    }
  }

  async getBridgeData(): Promise<TBridgeData | undefined> {
    try {
      const resources = await this.aptosClient.getAccountResources({
        accountAddress: BRIDGE_ADDRESS,
      });
      const bridgeResource = resources.find(
        (r) => r.type == `0x${BRIDGE_ADDRESS}::aptos_nft_bridge::Bridge`
      );
      return bridgeResource?.data as TBridgeData;
    } catch (error) {
      throw error;
    }
  }

  generateRandomSeed(length: number): string {
    let result = "";
    const characters =
      "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const charactersLength = characters.length;
    let counter = 0;
    while (counter < length) {
      result += characters.charAt(Math.floor(Math.random() * charactersLength));
      counter += 1;
    }
    return result;
  }

  convertToHexString(str: Uint8Array | string): string {
    return "0x" + Buffer.from(str).toString("hex");
  }
}
