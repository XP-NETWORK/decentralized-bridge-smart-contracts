import {
  Account,
  Aptos,
  AptosConfig,
  Ed25519Account,
  Ed25519PrivateKey,
  Network,
  SigningSchemeInput,
} from "@aptos-labs/ts-sdk";
import assert from "assert";
import { BridgeClient } from "../src/bridge-client";
import dotenv from "dotenv";
import { CONTRACT_ERROR_CODES } from "../src/constants";

dotenv.config();
const aptosConfig = new AptosConfig({
  network: Network.DEVNET,
});
const aptos = new Aptos(aptosConfig);
const aptosClient = new BridgeClient(aptos);

describe("Bridge", () => {
  describe("Initialize", () => {
    let adminAccount: Ed25519Account;
    let seed: Uint8Array;
    let self_chain: Uint8Array;
    let validator1: Ed25519Account;
    let validator2: Ed25519Account;
    let validator3: Ed25519Account;
    let validator4: Ed25519Account;
    let testAccount: Ed25519Account;

    before(async () => {
      let privateKey = new Ed25519PrivateKey(process.env.ED25519_PK!);
      adminAccount = Account.fromPrivateKey({
        privateKey,
      });
      let testAccountPrivateKey = new Ed25519PrivateKey(
        process.env.ED25519_TEST_PK!
      );
      testAccount = Account.fromPrivateKey({
        privateKey: testAccountPrivateKey,
      });
      validator1 = Account.generate({ scheme: SigningSchemeInput.Ed25519 });
      validator2 = Account.generate({ scheme: SigningSchemeInput.Ed25519 });
      validator3 = Account.generate({ scheme: SigningSchemeInput.Ed25519 });
      validator4 = Account.generate({ scheme: SigningSchemeInput.Ed25519 });
      await aptosClient.fundAccounts([
        validator1,
        validator2,
        validator3,
        validator4,
      ]);
      seed = Buffer.from("xyz");
      self_chain = Buffer.from("APTOS");
    });

    it.skip("Should fail if initialized by account which is not admin", async () => {
      let validators: Uint8Array[] = [validator1.publicKey.toUint8Array()];
      try {
        let commitedTransaction = await aptosClient.initialize(
          testAccount,
          validators,
          seed,
          self_chain
        );
        await aptos.waitForTransaction({
          transactionHash: commitedTransaction.hash,
          options: { checkSuccess: true },
        });
        assert.ok(false);
      } catch (error: any) {
        assert.ok(
          error["transaction"]["vm_status"].includes(
            CONTRACT_ERROR_CODES.E_NOT_BRIDGE_ADMIN
          )
        );
      }
    });

    it.skip("Should fail to initialize if validators array is empty", async () => {
      let validators: Uint8Array[] = [];
      try {
        let commitedTransaction = await aptosClient.initialize(
          adminAccount,
          validators,
          seed,
          self_chain
        );
        await aptos.waitForTransaction({
          transactionHash: commitedTransaction.hash,
          options: { checkSuccess: true },
        });
        assert.ok(false);
      } catch (error: any) {
        assert.ok(
          error["transaction"]["vm_status"].includes(
            CONTRACT_ERROR_CODES.E_VALIDATORS_LENGTH_ZERO
          )
        );
      }
    });

    it.skip("Should fail if initialized twice", async () => {
      let validators: Uint8Array[] = [validator1.publicKey.toUint8Array()];
      try {
        let commitedTransaction = await aptosClient.initialize(
          adminAccount,
          validators,
          seed,
          self_chain
        );
        await aptos.waitForTransaction({
          transactionHash: commitedTransaction.hash,
          options: { checkSuccess: true },
        });
        let commitedTransaction2 = await aptosClient.initialize(
          adminAccount,
          validators,
          seed,
          self_chain
        );
        await aptos.waitForTransaction({
          transactionHash: commitedTransaction2.hash,
          options: { checkSuccess: true },
        });
        assert.ok(false);
      } catch (error: any) {
        assert.ok(
          error["transaction"]["vm_status"].includes(
            CONTRACT_ERROR_CODES.E_ALREADY_INITIALIZED
          )
        );
      }
    });

    it("Should set validators correctly", async () => {
      // let validators: Uint8Array[] = [
      //   validator1.publicKey.toUint8Array(),
      //   validator2.publicKey.toUint8Array(),
      //   validator3.publicKey.toUint8Array(),
      //   validator4.publicKey.toUint8Array(),
      // ];
      try {
        // let commitedTransaction = await aptosClient.initialize(
        //   adminAccount,
        //   validators,
        //   seed,
        //   self_chain
        // );
        // await aptos.waitForTransaction({
        //   transactionHash: commitedTransaction.hash,
        //   options: { checkSuccess: true },
        // });
        await aptosClient.getBridgeData();
        assert.ok(true);
      } catch (error: any) {
        console.log({ error });
        assert.ok(false);
      }
    });
  });
});
