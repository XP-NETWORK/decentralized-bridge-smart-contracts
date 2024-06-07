import {
  Account,
  Aptos,
  AptosConfig,
  Ed25519Account,
  Ed25519PrivateKey,
  Network,
} from "@aptos-labs/ts-sdk";
import assert from "assert";
import { BridgeClient } from "../src/bridge-client";
import dotenv from "dotenv";
import { CHAIN_ID, CONTRACT_ERROR_CODES } from "../src/constants";
import * as ed from "@noble/ed25519";
import { BCS, HexString } from "aptos";
import { createHash } from "crypto";

dotenv.config();
const aptosConfig = new AptosConfig({
  network: Network.DEVNET,
});
const aptos = new Aptos(aptosConfig);
const aptosClient = new BridgeClient(aptos);

describe("Bridge", () => {
  let adminAccount: Ed25519Account;
  let seed: Uint8Array;
  let selfChain: Uint8Array;
  let validator1PbK: Uint8Array;
  let validator2PbK: Uint8Array;
  let validator3PbK: Uint8Array;
  let validator4PbK: Uint8Array;
  let validator5PbK: Uint8Array;
  let validator1PrK: Buffer;
  let validator2PrK: Buffer;
  let validator3PrK: Buffer;
  let validator4PrK: Buffer;
  let validator5PrK: Buffer;
  let testAccount: Ed25519Account;

  before(async () => {
    adminAccount = Account.fromPrivateKey({
      privateKey: new Ed25519PrivateKey(process.env.ED25519_PK!),
    });
    testAccount = Account.fromPrivateKey({
      privateKey: new Ed25519PrivateKey(process.env.ED25519_TEST_PK!),
    });
    validator1PbK = await ed.getPublicKey(process.env.VALIDATOR_1_PK!);
    validator2PbK = await ed.getPublicKey(process.env.VALIDATOR_2_PK!);
    validator3PbK = await ed.getPublicKey(process.env.VALIDATOR_3_PK!);
    validator4PbK = await ed.getPublicKey(process.env.VALIDATOR_4_PK!);
    validator5PbK = await ed.getPublicKey(process.env.VALIDATOR_5_PK!);
    validator1PrK = Buffer.from(process.env.VALIDATOR_1_PK!, "hex");
    validator2PrK = Buffer.from(process.env.VALIDATOR_2_PK!, "hex");
    validator3PrK = Buffer.from(process.env.VALIDATOR_3_PK!, "hex");
    validator4PrK = Buffer.from(process.env.VALIDATOR_4_PK!, "hex");
    validator5PrK = Buffer.from(process.env.VALIDATOR_5_PK!, "hex");
    seed = Buffer.from(aptosClient.generateRandomSeed(8));
    selfChain = Buffer.from(CHAIN_ID);
  });

  describe.skip("Initialize", () => {
    it("Should fail if initialized by account which is not admin", async () => {
      let validators: Uint8Array[] = [validator1PbK];
      try {
        let commitedTransaction = await aptosClient.initialize(
          testAccount,
          validators,
          seed,
          selfChain
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

    it("Should fail to initialize if validators array is empty", async () => {
      let validators: Uint8Array[] = [];
      try {
        let commitedTransaction = await aptosClient.initialize(
          adminAccount,
          validators,
          seed,
          selfChain
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

    // it("Should fail if initialized twice", async () => {
    //   let validators: Uint8Array[] = [validator1];
    //   try {
    //     let commitedTransaction = await aptosClient.initialize(
    //       adminAccount,
    //       validators,
    //       seed,
    //       selfChain
    //     );
    //     await aptos.waitForTransaction({
    //       transactionHash: commitedTransaction.hash,
    //       options: { checkSuccess: true },
    //     });
    //     let commitedTransaction2 = await aptosClient.initialize(
    //       adminAccount,
    //       validators,
    //       seed,
    //       selfChain
    //     );
    //     await aptos.waitForTransaction({
    //       transactionHash: commitedTransaction2.hash,
    //       options: { checkSuccess: true },
    //     });
    //     assert.ok(false);
    //   } catch (error: any) {
    //     assert.ok(
    //       error["transaction"]["vm_status"].includes(
    //         CONTRACT_ERROR_CODES.E_ALREADY_INITIALIZED
    //       )
    //     );
    //   }
    // });

    it("Should set bridge state correctly", async () => {
      let validatorsWithPublicKeys: string[] = [
        aptosClient.convertToHexString(validator1PbK),
        aptosClient.convertToHexString(validator2PbK),
        aptosClient.convertToHexString(validator3PbK),
        aptosClient.convertToHexString(validator4PbK),
      ];

      let validators: Uint8Array[] = [
        validator1PbK,
        validator2PbK,
        validator3PbK,
        validator4PbK,
      ];
      try {
        let commitedTransaction = await aptosClient.initialize(
          adminAccount,
          validators,
          seed,
          selfChain
        );
        await aptos.waitForTransaction({
          transactionHash: commitedTransaction.hash,
          options: { checkSuccess: true },
        });
        const response = await aptosClient.getBridgeData();
        if (response) {
          let contractValidators = response.validators.data;

          assert.ok(contractValidators.length === validators.length);

          contractValidators.forEach((validator, index) => {
            assert.ok(
              validator.key == validatorsWithPublicKeys[index] &&
                validator.value.pending_reward == "0"
            );
          });

          assert.ok(
            aptosClient.convertToHexString(CHAIN_ID) == response.self_chain
          );

          assert.ok(response.signer_cap?.account);
        }
        assert.ok(true);
      } catch (error: any) {
        console.log({ error });
        assert.ok(false);
      }
    });
  });

  describe.skip("Add Validator", async () => {
    it("Should fail if bridge is not intialized", async () => {
      const response = await aptosClient.getBridgeData();
      if (response?.validators) {
        assert.ok(true);
      } else {
        assert.ok(false);
      }
    });

    it("Should fail if signatures are not provided", async () => {
      try {
        let commitedTransaction = await aptosClient.addValidator(
          adminAccount,
          validator1PbK,
          [],
          []
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

    it("Should fail if public keys and signatures length is not equal", async () => {
      const newValidator = validator5PbK;

      const serializer = new BCS.Serializer();
      serializer.serializeFixedBytes(newValidator);
      const msgHash = createHash("SHA256")
        .update(serializer.getBytes())
        .digest();

      const validator1Signature = await ed.sign(msgHash, validator1PrK);

      try {
        let commitedTransaction = await aptosClient.addValidator(
          adminAccount,
          newValidator,
          [validator1Signature],
          [validator1PbK, validator2PbK]
        );
        await aptos.waitForTransaction({
          transactionHash: commitedTransaction.hash,
          options: { checkSuccess: true },
        });
        assert.ok(false);
      } catch (error: any) {
        assert.ok(
          error["transaction"]["vm_status"].includes(
            CONTRACT_ERROR_CODES.E_SIGNATURES_PUBLIC_KEYS_LENGTH_NOT_SAME
          )
        );
      }
    });

    it("Should fail if validator already exists", async () => {
      const alreadyAddedValidator = validator1PbK;
      const serializer = new BCS.Serializer();
      serializer.serializeFixedBytes(alreadyAddedValidator);
      const msgHash = createHash("SHA256")
        .update(serializer.getBytes())
        .digest();

      const validator1Signature = await ed.sign(msgHash, validator1PrK);
      const validator2Signature = await ed.sign(msgHash, validator2PrK);

      try {
        let commitedTransaction = await aptosClient.addValidator(
          adminAccount,
          alreadyAddedValidator,
          [validator1Signature, validator2Signature],
          [validator1PbK, validator2PbK]
        );
        await aptos.waitForTransaction({
          transactionHash: commitedTransaction.hash,
          options: { checkSuccess: true },
        });
        assert.ok(false);
      } catch (error: any) {
        assert.ok(
          error["transaction"]["vm_status"].includes(
            CONTRACT_ERROR_CODES.E_VALIDATOR_ALREADY_EXIST
          )
        );
      }
    });

    it("Should fail if thershold is not meet", async () => {
      const newValidator = validator5PbK;
      const serializer = new BCS.Serializer();
      serializer.serializeFixedBytes(newValidator);
      const msgHash = createHash("SHA256")
        .update(serializer.getBytes())
        .digest();

      const validator1Signature = await ed.sign(msgHash, validator1PrK);
      const validator2Signature = await ed.sign(msgHash, validator2PrK);

      try {
        let commitedTransaction = await aptosClient.addValidator(
          adminAccount,
          newValidator,
          [validator1Signature, validator2Signature],
          [validator1PbK, validator2PbK]
        );
        await aptos.waitForTransaction({
          transactionHash: commitedTransaction.hash,
          options: { checkSuccess: true },
        });
        assert.ok(false);
      } catch (error: any) {
        assert.ok(
          error["transaction"]["vm_status"].includes(
            CONTRACT_ERROR_CODES.E_THERSHOLD_NOT_REACHED
          )
        );
      }
    });

    it("Should fail if signatures are not valid", async () => {
      const newValidator = validator5PbK;
      const serializer = new BCS.Serializer();
      serializer.serializeFixedBytes(validator1PbK); // hashing wrong data. validator1 instead of newValidator
      const msgHash = createHash("SHA256")
        .update(serializer.getBytes())
        .digest();

      const validator1Signature = await ed.sign(msgHash, validator1PrK);
      const validator2Signature = await ed.sign(msgHash, validator2PrK);
      const validator3Signature = await ed.sign(msgHash, validator3PrK);
      const validator4Signature = await ed.sign(msgHash, validator4PrK);

      try {
        let commitedTransaction = await aptosClient.addValidator(
          adminAccount,
          newValidator,
          [
            validator1Signature,
            validator2Signature,
            validator3Signature,
            validator4Signature,
          ],
          [validator1PbK, validator2PbK, validator3PbK, validator4PbK]
        );
        await aptos.waitForTransaction({
          transactionHash: commitedTransaction.hash,
          options: { checkSuccess: true },
        });
        assert.ok(false);
      } catch (error: any) {
        assert.ok(
          error["transaction"]["vm_status"].includes(
            CONTRACT_ERROR_CODES.E_INVALID_SIGNATURE
          )
        );
      }
    });

    it("Should successfully add new validator", async () => {
      const newValidator = validator5PbK;

      const serializer = new BCS.Serializer();
      serializer.serializeFixedBytes(newValidator);
      const msgHash = createHash("SHA256")
        .update(serializer.getBytes())
        .digest();

      const validator1Signature = await ed.sign(msgHash, validator1PrK);
      const validator2Signature = await ed.sign(msgHash, validator2PrK);
      const validator3Signature = await ed.sign(msgHash, validator3PrK);
      const validator4Signature = await ed.sign(msgHash, validator4PrK);

      let validatorsWithPublicKeys: string[] = [
        aptosClient.convertToHexString(validator1PbK),
        aptosClient.convertToHexString(validator2PbK),
        aptosClient.convertToHexString(validator3PbK),
        aptosClient.convertToHexString(validator4PbK),
        aptosClient.convertToHexString(validator5PbK),
      ];

      try {
        let commitedTransaction = await aptosClient.addValidator(
          adminAccount,
          newValidator,
          [
            validator1Signature,
            validator2Signature,
            validator3Signature,
            validator4Signature,
          ],
          [validator1PbK, validator2PbK, validator3PbK, validator4PbK]
        );
        await aptos.waitForTransaction({
          transactionHash: commitedTransaction.hash,
          options: { checkSuccess: true },
        });
        const response = await aptosClient.getBridgeData();
        if (response) {
          let contractValidators = response.validators.data;

          assert.ok(contractValidators.length === 5);

          contractValidators.forEach((validator, index) => {
            assert.ok(
              validator.key == validatorsWithPublicKeys[index] &&
                validator.value.pending_reward == "0"
            );
          });
        }
        assert.ok(true);
      } catch (error: any) {
        console.log({ error });
        assert.ok(false);
      }
    });
  });
});
