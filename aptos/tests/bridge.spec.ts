import {
  Account,
  Aptos,
  AptosConfig,
  Ed25519Account,
  Ed25519PrivateKey,
  Network,
} from "@aptos-labs/ts-sdk";
import assert from "assert";
import { BridgeClient, TClaimData } from "../src/bridge-client";
import dotenv from "dotenv";
import {
  CHAIN_ID,
  CLAIM_FEE_5_APT,
  CLAIM_FEE_POINT_1_APT,
  CONTRACT_ERROR_CODES,
} from "../src/constants";
import * as ed from "@noble/ed25519";
import { BCS, HexString } from "aptos";
import { createHash } from "crypto";

dotenv.config();
const aptosConfig = new AptosConfig({
  network: Network.DEVNET,
});
const aptos = new Aptos(aptosConfig);

const aptosClient = new BridgeClient(aptos);

type TValidatorBalances = { [key: string]: string };

describe("Bridge", async () => {
  // TODO: Test with 100 signatures/validators

  let adminAccount: Ed25519Account;
  let testAccount: Ed25519Account;
  let nftOwner: Ed25519Account;
  let validator1: Ed25519Account;
  let seed: Uint8Array;
  let selfChain: Uint8Array;
  let validator1PbK: Uint8Array;
  let nftOwnerPbK: Uint8Array;
  let validator2PbK: Uint8Array;
  let validator3PbK: Uint8Array;
  let validator4PbK: Uint8Array;
  let validator5PbK: Uint8Array;
  let nftOwnerPrK: Buffer;
  let validator1PrK: Buffer;
  let validator2PrK: Buffer;
  let validator3PrK: Buffer;
  let validator4PrK: Buffer;
  let validator5PrK: Buffer;

  adminAccount = Account.fromPrivateKey({
    privateKey: new Ed25519PrivateKey(process.env.ED25519_PK!),
  });
  testAccount = Account.fromPrivateKey({
    privateKey: new Ed25519PrivateKey(process.env.ED25519_TEST_PK!),
  });
  nftOwner = Account.fromPrivateKey({
    privateKey: new Ed25519PrivateKey(process.env.NFT_OWNER_PK!),
  });
  validator1 = Account.fromPrivateKey({
    privateKey: new Ed25519PrivateKey(process.env.VALIDATOR_1_PK!),
  });
  validator1PbK = await ed.getPublicKey(process.env.VALIDATOR_1_PK!);
  validator2PbK = await ed.getPublicKey(process.env.VALIDATOR_2_PK!);
  validator3PbK = await ed.getPublicKey(process.env.VALIDATOR_3_PK!);
  validator4PbK = await ed.getPublicKey(process.env.VALIDATOR_4_PK!);
  validator5PbK = await ed.getPublicKey(process.env.VALIDATOR_5_PK!);
  nftOwnerPbK = await ed.getPublicKey(process.env.NFT_OWNER_PK!);
  validator1PrK = Buffer.from(process.env.VALIDATOR_1_PK!, "hex");
  validator2PrK = Buffer.from(process.env.VALIDATOR_2_PK!, "hex");
  validator3PrK = Buffer.from(process.env.VALIDATOR_3_PK!, "hex");
  validator4PrK = Buffer.from(process.env.VALIDATOR_4_PK!, "hex");
  validator5PrK = Buffer.from(process.env.VALIDATOR_5_PK!, "hex");
  nftOwnerPrK = Buffer.from(process.env.NFT_OWNER_PK!, "hex");
  seed = Buffer.from(aptosClient.generateRandomSeed(8));
  selfChain = Buffer.from(CHAIN_ID);

  describe.skip("Initialize", async () => {
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

  describe("After Initilazed", async () => {
    it("Should fail if bridge is not intialized", async () => {
      try {
        const response = await aptosClient.getBridgeData();
        if (response?.validators) {
          assert.ok(true);
        } else {
          assert.ok(false);
        }
      } catch (error) {
        assert.ok(false);
      }
    });

    describe.skip("Add Validator", async () => {
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

    describe("Lock", async () => {
      const collectionName = "Bridge";
      const destinationChain = Buffer.from("APTOS");
      const tokenSymbol = "ABC";
      const tokenName = "Bridge # 74";
      const tokenName1155 = "Bridge # 75";
      const tokenNameInvalid = "ABC _ 01";
      const collectionDescription = "ABC Fungible Collection Description";
      const collectionUri =
        "https://www.jumpstartmag.com/wp-content/uploads/2022/04/What-Is-NFT-Bridging.jpg";
      const tokenDescription = "Token Description";
      const tokenUri =
        "https://images.unsplash.com/photo-1643408875993-d7566153dd89?q=80&w=1780&auto=format&fit=crop";
      const sourceNftContractAddress = Buffer.from(
        "0xba92cf00f301b9fa4cf5ead497d128bdb3e05e1b"
      );
      let amount = 2;

      describe.skip("Lock 721", async () => {
        before(async () => {
          try {
            let commitedTransaction721 = await aptosClient.mintNft721(
              nftOwner,
              collectionName,
              collectionDescription,
              collectionUri,
              tokenName,
              tokenDescription,
              tokenUri
            );
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction721.hash,
              options: { checkSuccess: true },
            });
          } catch (error) {
            console.log({ error });
          }
        });

        it("Should fail if caller is not the owner", async () => {
          try {
            let commitedTransaction = await aptosClient.lock721(
              testAccount,
              collectionName,
              tokenName,
              destinationChain,
              1,
              sourceNftContractAddress
            );
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });
            assert.ok(false);
          } catch (error: any) {
            assert.ok(
              error["transaction"]["vm_status"].includes(
                CONTRACT_ERROR_CODES.EOBJECT_DOES_NOT_EXIST
              )
            );
          }
        });

        it("Should fail if token is not valid", async () => {
          try {
            let commitedTransaction = await aptosClient.lock721(
              nftOwner,
              collectionName,
              tokenNameInvalid,
              destinationChain,
              1,
              sourceNftContractAddress
            );
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });
            assert.ok(false);
          } catch (error: any) {
            assert.ok(
              error["transaction"]["vm_status"].includes(
                CONTRACT_ERROR_CODES.EOBJECT_DOES_NOT_EXIST
              )
            );
          }
        });

        it("Should successfully locks nft", async () => {
          try {
            const response = await aptosClient.userOwnsNft(
              new HexString(nftOwner.accountAddress.toString()),
              collectionName,
              tokenName
            );
            assert.ok(response[0]);

            let commitedTransaction = await aptosClient.lock721(
              nftOwner,
              collectionName,
              tokenName,
              destinationChain,
              1,
              sourceNftContractAddress
            );
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });
            assert.ok(true);
          } catch (error: any) {
            console.log({ error });
            assert.ok(false);
          }
        });
      });

      describe("Lock 1155", async () => {
        before(async () => {
          try {
            let commitedTransaction1155 = await aptosClient.mintNft1155(
              nftOwner,
              collectionName,
              collectionDescription,
              collectionUri,
              tokenName1155,
              tokenDescription,
              tokenUri,
              tokenSymbol,
              amount,
              collectionUri,
              "https://ufc.com"
            );
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction1155.hash,
              options: { checkSuccess: true },
            });
          } catch (error) {
            console.log({ error });
          }
        });

        it("Should fail if caller is not the owner", async () => {
          try {
            let commitedTransaction = await aptosClient.lock1155(
              testAccount,
              collectionName,
              tokenName1155,
              amount,
              destinationChain,
              1,
              sourceNftContractAddress
            );
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });
            assert.ok(false);
          } catch (error: any) {
            assert.ok(
              error["transaction"]["vm_status"].includes(
                CONTRACT_ERROR_CODES.EOBJECT_DOES_NOT_EXIST
              )
            );
          }
        });

        it("Should fail if token is not valid", async () => {
          try {
            let commitedTransaction = await aptosClient.lock1155(
              nftOwner,
              collectionName,
              tokenNameInvalid,
              amount,
              destinationChain,
              1,
              sourceNftContractAddress
            );
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });
            assert.ok(false);
          } catch (error: any) {
            assert.ok(
              error["transaction"]["vm_status"].includes(
                CONTRACT_ERROR_CODES.EOBJECT_DOES_NOT_EXIST
              )
            );
          }
        });

        it("Should fail if amount is zero", async () => {
          try {
            let commitedTransaction = await aptosClient.lock1155(
              nftOwner,
              collectionName,
              tokenName1155,
              0,
              destinationChain,
              1,
              sourceNftContractAddress
            );
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });
            assert.ok(false);
          } catch (error: any) {
            assert.ok(
              error["transaction"]["vm_status"].includes(
                CONTRACT_ERROR_CODES.E_TOKEN_AMOUNT_IS_ZERO
              )
            );
          }
        });

        it("Should successfully locks nft", async () => {
          try {
            let commitedTransaction = await aptosClient.lock1155(
              nftOwner,
              collectionName,
              tokenName1155,
              amount,
              destinationChain,
              1,
              sourceNftContractAddress
            );
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });
            assert.ok(true);
          } catch (error: any) {
            console.log({ error });
            assert.ok(false);
          }
        });
      });
    });

    describe.skip("Claim", async () => {
      describe("Claim 721", async () => {
        let claimData: TClaimData = {
          collection: "Bridge",
          tokenId: "63",
          description: "Token Description",
          uri: "https://upload.wikimedia.org/wikipedia/en/thumb/8/89/2024_ICC_Men%27s_T20_World_Cup_logo.svg/1200px-2024_ICC_Men%27s_T20_World_Cup_logo.svg.png",
          royaltyPointsNumerator: 1,
          royaltyPointsDenominator: 5,
          royaltyPayeeAddress: new HexString(
            nftOwner.accountAddress.toString()
          ),
          fee: CLAIM_FEE_5_APT, // in octas. 1 APT = 10 ^ 8 octas
          destinationChain: Buffer.from("BSC"),
          sourceChain: Buffer.from("APTOS"),
          sourceNftContractAddress: Buffer.from(
            "0xba92cf00f301b9fa4cf5ead497d128bdb3e05e1b"
          ),
          transactionHash: Buffer.from(
            "0x9724e4d237117018e5d2135036d879b25ca36ae4469120b85ef7ebba8fa408d5"
          ),
          nftType: Buffer.from("multiple"), // singular for 721, multiple for 1155
          symbol: "WCC",
          amount: 0, // set this to 0 for claim 721
          iconUri:
            "https://upload.wikimedia.org/wikipedia/en/thumb/8/89/2024_ICC_Men%27s_T20_World_Cup_logo.svg/1200px-2024_ICC_Men%27s_T20_World_Cup_logo.svg.png",
          projectUri:
            "https://images.icc-cricket.com/image/private/t_q-best/v1706276260/prd/assets/tournaments/t20worldcup/2024/t20-logo-horizontal-light.svg",
          metadata: "asdf",
          signatures: [],
          publicKeys: [validator1PbK, validator2PbK],
          sender: nftOwner,
        };

        it("Should fail if nft type is not valid", async () => {
          const msgHash = aptosClient.generateClaimDataHash(
            claimData,
            nftOwner
          );

          claimData.signatures = await Promise.all([
            ed.sign(msgHash, validator1PrK),
            ed.sign(msgHash, validator2PrK),
          ]);

          try {
            let commitedTransaction = await aptosClient.claim721(claimData);
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });
            assert.ok(false);
          } catch (error: any) {
            assert.ok(
              error["transaction"]["vm_status"].includes(
                CONTRACT_ERROR_CODES.E_INVALID_NFT_TYPE
              )
            );
          }
        });

        it("Should fail if destination chain is not valid", async () => {
          claimData.nftType = Buffer.from("singular");
          const msgHash = aptosClient.generateClaimDataHash(
            claimData,
            nftOwner
          );

          claimData.signatures = await Promise.all([
            ed.sign(msgHash, validator1PrK),
            ed.sign(msgHash, validator2PrK),
          ]);

          try {
            let commitedTransaction = await aptosClient.claim721(claimData);
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });
            assert.ok(false);
          } catch (error: any) {
            assert.ok(
              error["transaction"]["vm_status"].includes(
                CONTRACT_ERROR_CODES.E_INVALID_DESTINATION_CHAIN
              )
            );
          }
        });

        it("Should fail if thershold is not meet", async () => {
          claimData.destinationChain = Buffer.from("APTOS");

          const msgHash = aptosClient.generateClaimDataHash(
            claimData,
            nftOwner
          );

          claimData.signatures = await Promise.all([
            ed.sign(msgHash, validator1PrK),
            ed.sign(msgHash, validator2PrK),
          ]);

          try {
            let commitedTransaction = await aptosClient.claim721(claimData);
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

        it("Should fail if user balance is less than fees", async () => {
          claimData.publicKeys.push(validator3PbK);

          const msgHash = aptosClient.generateClaimDataHash(
            claimData,
            nftOwner
          );

          claimData.signatures = await Promise.all([
            ed.sign(msgHash, validator1PrK),
            ed.sign(msgHash, validator2PrK),
            ed.sign(msgHash, validator3PrK),
          ]);

          try {
            let commitedTransaction = await aptosClient.claim721(claimData);
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });
            assert.ok(false);
          } catch (error: any) {
            assert.ok(
              error["transaction"]["vm_status"].includes(
                CONTRACT_ERROR_CODES.EINSUFFICIENT_BALANCE
              )
            );
          }
        });

        it("Should be able to claim and mint new nft successfully ", async () => {
          claimData.fee = CLAIM_FEE_POINT_1_APT;
          const msgHash = aptosClient.generateClaimDataHash(
            claimData,
            nftOwner
          );

          claimData.signatures = await Promise.all([
            ed.sign(msgHash, validator1PrK),
            ed.sign(msgHash, validator2PrK),
            ed.sign(msgHash, validator3PrK),
          ]);

          try {
            // const nftOwnerBalanceBeforeTx = await aptos.getAccountAPTAmount({
            //   accountAddress: nftOwner.accountAddress,
            // });

            let validatorBalances: TValidatorBalances = {};
            const bridgeDataBeforeTx = await aptosClient.getBridgeData();

            if (bridgeDataBeforeTx) {
              let contractValidators = bridgeDataBeforeTx.validators.data;
              contractValidators.forEach((validator, index) => {
                validatorBalances[validator.key] =
                  validator.value.pending_reward;
              });
            } else {
              assert.ok(false);
            }

            let commitedTransaction = await aptosClient.claim721(claimData);
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });

            // const nftOwnerBalanceAfterTx = await aptos.getAccountAPTAmount({
            //   accountAddress: nftOwner.accountAddress,
            // });

            // assert.ok(
            //   nftOwnerBalanceAfterTx <= nftOwnerBalanceBeforeTx - claimData.fee
            // );

            const bridgeDataAfterTx = await aptosClient.getBridgeData();
            const rewardPerValidator =
              claimData.fee / claimData.publicKeys.length;

            if (bridgeDataAfterTx) {
              let contractValidators = bridgeDataAfterTx.validators.data;
              contractValidators.forEach((validator, index) => {
                if (!validatorBalances[validator.key]) {
                  assert.ok(true);
                } else {
                  assert.ok(
                    (validatorBalances[validator.key] = (
                      Number(validator.value.pending_reward) +
                      rewardPerValidator
                    ).toString())
                  );
                }
              });
            } else {
              assert.ok(false);
            }
          } catch (error: any) {
            console.log({ error });
            assert.ok(false);
          }
        });

        it.skip("Should be able to claim and unlock nft successfully ", async () => {
          claimData.fee = CLAIM_FEE_POINT_1_APT;
          claimData.tokenId = "45";
          const msgHash = aptosClient.generateClaimDataHash(
            claimData,
            nftOwner
          );

          claimData.signatures = await Promise.all([
            ed.sign(msgHash, validator1PrK),
            ed.sign(msgHash, validator2PrK),
            ed.sign(msgHash, validator3PrK),
          ]);

          try {
            const nftOwnerBalanceBeforeTx = await aptos.getAccountAPTAmount({
              accountAddress: nftOwner.accountAddress,
            });

            let validatorBalances: TValidatorBalances = {};
            const bridgeDataBeforeTx = await aptosClient.getBridgeData();

            if (bridgeDataBeforeTx) {
              let contractValidators = bridgeDataBeforeTx.validators.data;
              contractValidators.forEach((validator, index) => {
                validatorBalances[validator.key] =
                  validator.value.pending_reward;
              });
            } else {
              assert.ok(false);
            }

            let commitedTransaction = await aptosClient.claim721(claimData);
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });

            const nftOwnerBalanceAfterTx = await aptos.getAccountAPTAmount({
              accountAddress: nftOwner.accountAddress,
            });

            assert.ok(
              nftOwnerBalanceAfterTx <= nftOwnerBalanceBeforeTx - claimData.fee
            );

            const bridgeDataAfterTx = await aptosClient.getBridgeData();
            const rewardPerValidator =
              claimData.fee / claimData.publicKeys.length;

            if (bridgeDataAfterTx) {
              let contractValidators = bridgeDataAfterTx.validators.data;
              contractValidators.forEach((validator, index) => {
                if (!validatorBalances[validator.key]) {
                  assert.ok(true);
                } else {
                  assert.ok(
                    (validatorBalances[validator.key] = (
                      Number(validator.value.pending_reward) +
                      rewardPerValidator
                    ).toString())
                  );
                }
              });
            } else {
              assert.ok(false);
            }

            const response = await aptosClient.userOwnsNft(
              new HexString(nftOwner.accountAddress.toString()),
              claimData.collection,
              `${claimData.collection} # ${claimData.tokenId}`
            );
            assert.ok(response[0]);
          } catch (error: any) {
            console.log({ error });
            // console.log({ error: error.transaction.payload.arguments });
            assert.ok(false);
          }
        });
      });

      describe.skip("Claim 1155", async () => {
        let claimData: TClaimData = {
          collection: "Bridge",
          tokenId: "47S",
          description: "Token Description",
          uri: "https://images.unsplash.com/photo-1643408875993-d7566153dd89?q=80&w=1780&auto=format&fit=crop",
          royaltyPointsNumerator: 1,
          royaltyPointsDenominator: 5,
          royaltyPayeeAddress: new HexString(
            nftOwner.accountAddress.toString()
          ),
          fee: CLAIM_FEE_5_APT, // in octas. 1 APT = 10 ^ 8 octas
          destinationChain: Buffer.from("BSC"),
          sourceChain: Buffer.from("APTOS"),
          sourceNftContractAddress: Buffer.from(
            "0xba92cf00f301b9fa4cf5ead497d128bdb3e05e1b"
          ),
          transactionHash: Buffer.from(
            "0x9724e4d237117018e5d2135036d879b25ca36ae4469120b85ef7ebba8fa408d5"
          ),
          nftType: Buffer.from("singular"), // singular for 721, multiple for 1155
          symbol: "WCC",
          amount: 5, // set this to 0 for claim 721
          iconUri:
            "https://www.jumpstartmag.com/wp-content/uploads/2022/04/What-Is-NFT-Bridging.jpg",
          projectUri:
            "https://www.jumpstartmag.com/wp-content/uploads/2022/04/What-Is-NFT-Bridging.jpg",
          metadata: "asdf",
          signatures: [],
          publicKeys: [validator1PbK, validator2PbK],
          sender: nftOwner,
        };

        it("Should fail if nft type is not valid", async () => {
          const msgHash = aptosClient.generateClaimDataHash(
            claimData,
            nftOwner
          );

          claimData.signatures = await Promise.all([
            ed.sign(msgHash, validator1PrK),
            ed.sign(msgHash, validator2PrK),
          ]);

          try {
            let commitedTransaction = await aptosClient.claim1155(claimData);
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });
            assert.ok(false);
          } catch (error: any) {
            assert.ok(
              error["transaction"]["vm_status"].includes(
                CONTRACT_ERROR_CODES.E_INVALID_NFT_TYPE
              )
            );
          }
        });

        it("Should fail if destination chain is not valid", async () => {
          claimData.nftType = Buffer.from("multiple");
          const msgHash = aptosClient.generateClaimDataHash(
            claimData,
            nftOwner
          );

          claimData.signatures = await Promise.all([
            ed.sign(msgHash, validator1PrK),
            ed.sign(msgHash, validator2PrK),
          ]);

          try {
            let commitedTransaction = await aptosClient.claim1155(claimData);
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });
            assert.ok(false);
          } catch (error: any) {
            assert.ok(
              error["transaction"]["vm_status"].includes(
                CONTRACT_ERROR_CODES.E_INVALID_DESTINATION_CHAIN
              )
            );
          }
        });

        it("Should fail if thershold is not meet", async () => {
          claimData.destinationChain = Buffer.from("APTOS");

          const msgHash = aptosClient.generateClaimDataHash(
            claimData,
            nftOwner
          );

          claimData.signatures = await Promise.all([
            ed.sign(msgHash, validator1PrK),
            ed.sign(msgHash, validator2PrK),
          ]);

          try {
            let commitedTransaction = await aptosClient.claim1155(claimData);
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

        it("Should fail if user balance is less than fees", async () => {
          claimData.publicKeys.push(validator3PbK);

          const msgHash = aptosClient.generateClaimDataHash(
            claimData,
            nftOwner
          );

          claimData.signatures = await Promise.all([
            ed.sign(msgHash, validator1PrK),
            ed.sign(msgHash, validator2PrK),
            ed.sign(msgHash, validator3PrK),
          ]);

          try {
            let commitedTransaction = await aptosClient.claim1155(claimData);
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });
            assert.ok(false);
          } catch (error: any) {
            assert.ok(
              error["transaction"]["vm_status"].includes(
                CONTRACT_ERROR_CODES.EINSUFFICIENT_BALANCE
              )
            );
          }
        });

        it.skip("Should be able to claim and mint new nft successfully ", async () => {
          claimData.fee = CLAIM_FEE_POINT_1_APT;
          claimData.nftType = Buffer.from("multiple");

          const msgHash = aptosClient.generateClaimDataHash(
            claimData,
            nftOwner
          );

          claimData.signatures = await Promise.all([
            ed.sign(msgHash, validator1PrK),
            ed.sign(msgHash, validator2PrK),
            ed.sign(msgHash, validator3PrK),
          ]);

          try {
            const nftOwnerBalanceBeforeTx = await aptos.getAccountAPTAmount({
              accountAddress: nftOwner.accountAddress,
            });

            let validatorBalances: TValidatorBalances = {};
            const bridgeDataBeforeTx = await aptosClient.getBridgeData();

            if (bridgeDataBeforeTx) {
              let contractValidators = bridgeDataBeforeTx.validators.data;
              contractValidators.forEach((validator, index) => {
                validatorBalances[validator.key] =
                  validator.value.pending_reward;
              });
            } else {
              assert.ok(false);
            }

            let commitedTransaction = await aptosClient.claim1155(claimData);
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });

            const nftOwnerBalanceAfterTx = await aptos.getAccountAPTAmount({
              accountAddress: nftOwner.accountAddress,
            });

            assert.ok(
              nftOwnerBalanceAfterTx <= nftOwnerBalanceBeforeTx - claimData.fee
            );

            const bridgeDataAfterTx = await aptosClient.getBridgeData();
            const rewardPerValidator =
              claimData.fee / claimData.publicKeys.length;

            if (bridgeDataAfterTx) {
              let contractValidators = bridgeDataAfterTx.validators.data;
              contractValidators.forEach((validator, index) => {
                if (!validatorBalances[validator.key]) {
                  assert.ok(true);
                } else {
                  assert.ok(
                    (validatorBalances[validator.key] = (
                      Number(validator.value.pending_reward) +
                      rewardPerValidator
                    ).toString())
                  );
                }
              });
            } else {
              assert.ok(false);
            }
          } catch (error: any) {
            console.log({ error });
            assert.ok(false);
          }
        });

        it("Should be able to claim and unlock nft successfully ", async () => {
          claimData.fee = CLAIM_FEE_POINT_1_APT;
          claimData.nftType = Buffer.from("multiple");
          claimData.tokenId = "47";

          const msgHash = aptosClient.generateClaimDataHash(
            claimData,
            nftOwner
          );

          claimData.signatures = await Promise.all([
            ed.sign(msgHash, validator1PrK),
            ed.sign(msgHash, validator2PrK),
            ed.sign(msgHash, validator3PrK),
          ]);

          try {
            const nftOwnerBalanceBeforeTx = await aptos.getAccountAPTAmount({
              accountAddress: nftOwner.accountAddress,
            });

            let validatorBalances: TValidatorBalances = {};
            const bridgeDataBeforeTx = await aptosClient.getBridgeData();

            if (bridgeDataBeforeTx) {
              let contractValidators = bridgeDataBeforeTx.validators.data;
              contractValidators.forEach((validator, index) => {
                validatorBalances[validator.key] =
                  validator.value.pending_reward;
              });
            } else {
              assert.ok(false);
            }

            let commitedTransaction = await aptosClient.claim1155(claimData);
            await aptos.waitForTransaction({
              transactionHash: commitedTransaction.hash,
              options: { checkSuccess: true },
            });

            const nftOwnerBalanceAfterTx = await aptos.getAccountAPTAmount({
              accountAddress: nftOwner.accountAddress,
            });

            assert.ok(
              nftOwnerBalanceAfterTx <= nftOwnerBalanceBeforeTx - claimData.fee
            );

            // const response = await aptosClient.userOwnsNft(
            //   new HexString(nftOwner.accountAddress.toString()),
            //   claimData.collection,
            //   `${claimData.collection} # ${claimData.tokenId}`
            // );
            // assert.ok(response[0]);
            const bridgeDataAfterTx = await aptosClient.getBridgeData();
            const rewardPerValidator =
              claimData.fee / claimData.publicKeys.length;

            if (bridgeDataAfterTx) {
              let contractValidators = bridgeDataAfterTx.validators.data;
              contractValidators.forEach((validator, index) => {
                if (!validatorBalances[validator.key]) {
                  assert.ok(true);
                } else {
                  assert.ok(
                    (validatorBalances[validator.key] = (
                      Number(validator.value.pending_reward) +
                      rewardPerValidator
                    ).toString())
                  );
                }
              });
            } else {
              assert.ok(false);
            }
          } catch (error: any) {
            console.log({ error: error });
            // console.log({ error: error.transaction.payload.arguments });
            assert.ok(false);
          }
        });
      });
    });

    describe.skip("Claim Validator Reward", async () => {
      it("should fail if sender is not admin", async () => {
        try {
          const serializer = new BCS.Serializer();
          serializer.serializeFixedBytes(validator1PbK); // hashing wrong data. validator1 instead of newValidator
          const msgHash = createHash("SHA256")
            .update(serializer.getBytes())
            .digest();

          const validator1Signature = await ed.sign(msgHash, validator1PrK);
          const validator2Signature = await ed.sign(msgHash, validator2PrK);
          const validator3Signature = await ed.sign(msgHash, validator3PrK);
          const validator4Signature = await ed.sign(msgHash, validator4PrK);

          let commitedTransaction = await aptosClient.claimValidatorRewards(
            nftOwner,
            new HexString(nftOwner.accountAddress.toString()),
            validator1PbK,
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
              CONTRACT_ERROR_CODES.E_NOT_BRIDGE_ADMIN
            )
          );
        }
      });

      it("should fail if validator doesnot exist", async () => {
        try {
          const serializer = new BCS.Serializer();
          serializer.serializeFixedBytes(validator1PbK); // hashing wrong data. validator1 instead of newValidator
          const msgHash = createHash("SHA256")
            .update(serializer.getBytes())
            .digest();

          const validator1Signature = await ed.sign(msgHash, validator1PrK);
          const validator2Signature = await ed.sign(msgHash, validator2PrK);
          const validator3Signature = await ed.sign(msgHash, validator3PrK);
          const validator4Signature = await ed.sign(msgHash, validator4PrK);

          let commitedTransaction = await aptosClient.claimValidatorRewards(
            adminAccount,
            new HexString(nftOwner.accountAddress.toString()),
            validator5PbK,
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
              CONTRACT_ERROR_CODES.E_VALIDATOR_DOESNOT_EXIST
            )
          );
        }
      });

      it("should fail if thershold is not meet", async () => {
        try {
          const serializer = new BCS.Serializer();
          serializer.serializeFixedBytes(validator1PbK); // hashing wrong data. validator1 instead of newValidator
          const msgHash = createHash("SHA256")
            .update(serializer.getBytes())
            .digest();

          const validator1Signature = await ed.sign(msgHash, validator1PrK);
          const validator2Signature = await ed.sign(msgHash, validator2PrK);

          let commitedTransaction = await aptosClient.claimValidatorRewards(
            adminAccount,
            new HexString(nftOwner.accountAddress.toString()),
            validator1PbK,
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

      it.skip("should fail if validator pending reward is zero", async () => {
        try {
          const serializer = new BCS.Serializer();
          serializer.serializeFixedBytes(validator5PbK);
          const msgHash = createHash("SHA256")
            .update(serializer.getBytes())
            .digest();

          const validator1Signature = await ed.sign(msgHash, validator1PrK);
          const validator2Signature = await ed.sign(msgHash, validator2PrK);
          const validator3Signature = await ed.sign(msgHash, validator3PrK);
          const validator4Signature = await ed.sign(msgHash, validator4PrK);

          let commitedTransaction = await aptosClient.claimValidatorRewards(
            adminAccount,
            new HexString(validator1.accountAddress.toString()),
            validator5PbK,
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
              CONTRACT_ERROR_CODES.E_VALIDATOR_PENDING_REWARD_IS_ZERO
            )
          );
        }
      });

      it("should successfully transfer validator pending reward", async () => {
        try {
          const validatorToReward = validator1;
          const validatorToRewardPbK =
            validatorToReward.publicKey.toUint8Array();
          const serializer = new BCS.Serializer();
          serializer.serializeFixedBytes(validatorToRewardPbK); // hashing wrong data. validator1 instead of newValidator
          const msgHash = createHash("SHA256")
            .update(serializer.getBytes())
            .digest();

          const validator1Signature = await ed.sign(msgHash, validator1PrK);
          const validator2Signature = await ed.sign(msgHash, validator2PrK);
          const validator3Signature = await ed.sign(msgHash, validator3PrK);
          const validator4Signature = await ed.sign(msgHash, validator4PrK);

          const validatorBalanceBeforeTx = await aptos.getAccountAPTAmount({
            accountAddress: validatorToReward.accountAddress,
          });

          let validatorBalances: TValidatorBalances = {};

          const responseBefore = await aptosClient.getBridgeData();

          if (responseBefore) {
            let contractValidators = responseBefore.validators.data;
            contractValidators.forEach((validator, index) => {
              validatorBalances[validator.key] = validator.value.pending_reward;
            });
          }

          let commitedTransaction = await aptosClient.claimValidatorRewards(
            adminAccount,
            new HexString(validator1.accountAddress.toString()),
            validatorToRewardPbK,
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

          const validatorBalanceAfterTx = await aptos.getAccountAPTAmount({
            accountAddress: validatorToReward.accountAddress,
          });

          assert.ok(
            validatorBalanceAfterTx ===
              validatorBalanceBeforeTx + Math.trunc(CLAIM_FEE_POINT_1_APT / 3)
          );

          const response = await aptosClient.getBridgeData();

          if (response) {
            let contractValidators = response.validators.data;
            contractValidators.forEach((validator, index) => {
              if (validator.key === validatorToRewardPbK.toString()) {
                assert.ok(validator.value.pending_reward === "0");
              } else {
                assert.ok(
                  (validatorBalances[validator.key] =
                    validator.value.pending_reward)
                );
              }
            });
          }
        } catch (error: any) {
          console.log({ error });
          assert.ok(false);
        }
      });
    });
  });
});
