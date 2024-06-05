import {
  Account,
  AccountData,
  Aptos,
  AptosConfig,
  Ed25519Account,
  Network,
} from "@aptos-labs/ts-sdk";
import { BRIDGE_ADDRESS, BRIDGE_MODULE, BRIDGE_FUNCTIONS } from "./constants";

export class BridgeClient {
  private aptosClient: Aptos;

  constructor(client: Aptos) {
    this.aptosClient = client;
  }

  async fundAccounts(accounts: Ed25519Account[]) {
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
    account: Ed25519Account,
    validators: Uint8Array[],
    seed: Uint8Array,
    self_chain: Uint8Array
  ) {
    const transaction = await this.aptosClient.transaction.build.simple({
      sender: account.accountAddress,
      data: {
        function: `${BRIDGE_ADDRESS}::${BRIDGE_MODULE}::${BRIDGE_FUNCTIONS.Initialize}`,
        functionArguments: [validators, seed, self_chain],
      },
    });

    return this.aptosClient.signAndSubmitTransaction({
      signer: account,
      transaction,
    });
  }

  async getBridgeData() {
    const resources = await this.aptosClient.getAccountResources({
      accountAddress: BRIDGE_ADDRESS,
    });
    console.log({ resources });
    const accountResource = resources.find(
      (r) => r.type == `0x${BRIDGE_ADDRESS}::aptos_nft_bridge::Bridge`
    );
    console.log({
      accountResource: (accountResource?.data as any)?.validators,
    });
  }
}
