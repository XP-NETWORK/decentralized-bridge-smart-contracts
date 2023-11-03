import { expect } from "chai";
import { ethers } from "hardhat";
import ethersOriginal from "ethers";
import { BridgeStorage } from "../contractsTypes";
import {
    ContractTransactionReceipt,
    ContractTransactionResponse,
    HDNodeWallet,
} from "ethers";
import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";

describe("BridgeStorage", function () {
    let bridgeStorage: BridgeStorage & {
        deploymentTransaction(): ContractTransactionResponse;
    };
    let owner: HardhatEthersSigner,
        validator1: HardhatEthersSigner,
        validator2: HardhatEthersSigner,
        validator3: HardhatEthersSigner,
        validator4: HardhatEthersSigner;

    async function setBalance(address: string, amountInEther: string) {
        const provider = ethers.provider;

        const newBalance = ethers.parseEther(amountInEther);

        // this is necessary because hex quantities with leading zeros are not valid at the JSON-RPC layer
        const newBalanceHex = newBalance.toString().replace("0x0", "0x");
        await provider.send("hardhat_setBalance", [address, newBalanceHex]);
    }

    // async function addValidators(numNewValidators: number) {
    //     // `existingValidators` is an array of signer objects that have already been added to the system.
    //     // `numNewValidators` is the number of new validators you want to add.

    //     const newValidators: HDNodeWallet[] = []; // This will store the new validator signers.
    //     const approvalPromises: Promise<ContractTransactionReceipt | null>[] =
    //         []; // This will store the promises for the approval transactions.
    //     const existingValidators = [validator1];
    //     for (let i = 0; i < numNewValidators; i++) {
    //         // Generate or get a new validator account here. For this example, we'll pretend it's a new signer from ethers.
    //         const newValidator = ethers.Wallet.createRandom().connect(
    //             ethers.provider
    //         );

    //         await setBalance(newValidator.address, "5");

    //         // Generate or assign the signatures for approving the stake here.
    //         const signature = `signature${i + 1}`; // Replace this with actual signature generation or retrieval logic.

    //         // For each existing validator, add an approval transaction promise for the new validator.
    //         existingValidators.forEach((existingValidator, i) => {
    //             const approvalPromise = bridgeStorage
    //                 .connect(existingValidator)
    //                 .approveStake(newValidator.address, signature + i)
    //                 .then((r) => r.wait());
    //             approvalPromises.push(approvalPromise);
    //         });

    //         // Add the new validator to the list of existing validators for subsequent iterations.
    //         existingValidators.push(
    //             newValidator as unknown as HardhatEthersSigner
    //         );

    //         // Also add the new validator to the list of new validators to return.
    //         newValidators.push(newValidator);

    //         // Wait for all approval transactions for this new validator to be added before continuing.
    //         await Promise.all(approvalPromises);
    //     }

    //     // Return the array of new validators
    //     return existingValidators;
    // }

    beforeEach(async function () {
        [owner, validator1, validator2, validator3, validator4] =
            await ethers.getSigners();

        const BridgeStorage = await ethers.getContractFactory("BridgeStorage");
        bridgeStorage = await BridgeStorage.deploy(validator1.address, [
            { chain: "ETH", fee: 100 },
            { chain: "BSC", fee: 200 },
        ]);
    });

    it("should initialize the contract correctly", async function () {
        const [chainFee, validatorCount, validatorExists] = await Promise.all([
            bridgeStorage.chainFee("ETH"),
            bridgeStorage.validatorCount(),
            bridgeStorage.validators(validator1.address),
        ]);
        // await addValidators(2);
        expect(chainFee).to.equal(100);
        expect(validatorCount).to.equal(1);
        expect(validatorExists).to.be.eq(true);
    });

    describe("changeChainFee()", async function () {
        it("should change chain fee with 2/3 + 1 validator votes", async function () {
            // add 2 new validators
            await Promise.all([
                bridgeStorage
                    .connect(validator1)
                    .approveStake(validator2.address, "signature1"),

                bridgeStorage
                    .connect(validator1)
                    .approveStake(validator3.address, "signature2"),
            ]);

            await Promise.all([
                bridgeStorage.connect(validator1).changeChainFee("ETH", 150),
                bridgeStorage.connect(validator2).changeChainFee("ETH", 150),
            ]);

            const newChainFee = await bridgeStorage.chainFee("ETH");
            expect(newChainFee).to.equal(150);
        });

        it("should change chain fee with 2/3 + 1 validator votes and taking only the majority decided fee", async function () {
            // add 3 new validators
            await bridgeStorage
                .connect(validator1)
                .approveStake(validator2.address, "signature1")
                .then((r) => r.wait());

            await Promise.all([
                bridgeStorage
                    .connect(validator1)
                    .approveStake(validator3.address, "signature2")
                    .then((r) => r.wait()),

                bridgeStorage
                    .connect(validator2)
                    .approveStake(validator3.address, "signature3")
                    .then((r) => r.wait()),
            ]);

            await Promise.all([
                bridgeStorage
                    .connect(validator1)
                    .approveStake(validator4.address, "signature4")
                    .then((r) => r.wait()),

                bridgeStorage
                    .connect(validator2)
                    .approveStake(validator4.address, "signature5")
                    .then((r) => r.wait()),

                bridgeStorage
                    .connect(validator3)
                    .approveStake(validator4.address, "signature6")
                    .then((r) => r.wait()),
            ]);

            await Promise.all([
                bridgeStorage
                    .connect(validator1)
                    .changeChainFee("ETH", 180)
                    .then((r) => r.wait()),

                bridgeStorage
                    .connect(validator2)
                    .changeChainFee("ETH", 150)
                    .then((r) => r.wait()),

                bridgeStorage
                    .connect(validator3)
                    .changeChainFee("ETH", 180)
                    .then((r) => r.wait()),

                bridgeStorage
                    .connect(validator4)
                    .changeChainFee("ETH", 180)
                    .then((r) => r.wait()),
            ]);

            const newChainFee = await bridgeStorage.chainFee("ETH");
            expect(newChainFee).to.equal(180);
        });

        it("should not change chain fee if no proposed fee is able to get majority votes from validators", async function () {
            // add 3 new validators
            await bridgeStorage
                .connect(validator1)
                .approveStake(validator2.address, "signature1");

            await Promise.all([
                bridgeStorage
                    .connect(validator1)
                    .approveStake(validator3.address, "signature2"),

                bridgeStorage
                    .connect(validator2)
                    .approveStake(validator3.address, "signature3"),
            ]);

            await Promise.all([
                bridgeStorage
                    .connect(validator1)
                    .approveStake(validator4.address, "signature4"),
                bridgeStorage
                    .connect(validator2)
                    .approveStake(validator4.address, "signature5"),
                bridgeStorage
                    .connect(validator3)
                    .approveStake(validator4.address, "signature6"),
            ]);

            await Promise.all([
                bridgeStorage.connect(validator1).changeChainFee("ETH", 180),
                bridgeStorage.connect(validator2).changeChainFee("ETH", 150),
                bridgeStorage.connect(validator3).changeChainFee("ETH", 180),
                bridgeStorage.connect(validator4).changeChainFee("ETH", 181),
            ]);

            const newChainFee = await bridgeStorage.chainFee("ETH");
            expect(newChainFee).to.equal(100);
        });

        it("should not change chain fee with less than 2/3 + 1 validator votes", async function () {
            // add 2 new validators
            await Promise.all([
                bridgeStorage
                    .connect(validator1)
                    .approveStake(validator2.address, "signature1"),

                bridgeStorage
                    .connect(validator1)
                    .approveStake(validator3.address, "signature2"),
            ]);

            await bridgeStorage.connect(validator2).changeChainFee("ETH", 150);

            const newChainFee = await bridgeStorage.chainFee("ETH");
            expect(newChainFee).to.equal(100); // Chain fee should remain unchanged
        });

        it("should fail if already voted", async function () {
            await bridgeStorage
                .connect(validator1)
                .approveStake(validator2.address, "signature1");

            await Promise.all([
                bridgeStorage
                    .connect(validator1)
                    .approveStake(validator3.address, "signature2"),

                bridgeStorage
                    .connect(validator2)
                    .approveStake(validator3.address, "signature3"),
            ]);

            await bridgeStorage.connect(validator1).changeChainFee("ETH", 150);

            await expect(
                bridgeStorage.connect(validator1).changeChainFee("ETH", 150)
            ).to.be.revertedWith("Already voted");
        });

        it("should fail if not validator", async function () {
            await expect(
                bridgeStorage.connect(validator2).changeChainFee("ETH", 150)
            ).to.be.revertedWith("Only validators can call this function");
        });
    });

    describe("changeValidatorStatus()", async function () {
        it("should be able to change validator status true and false and validator count should change accordingly", async function () {
            // // increase validator count to 3
            await bridgeStorage
                .connect(validator1)
                .approveStake(validator2.address, "signature1");

            await Promise.all([
                bridgeStorage
                    .connect(validator1)
                    .approveStake(validator3.address, "signature2"),

                bridgeStorage
                    .connect(validator2)
                    .approveStake(validator3.address, "signature3"),
            ]);

            await Promise.all([
                bridgeStorage
                    .connect(validator1)
                    .approveStake(validator4.address, "signature4"),

                bridgeStorage
                    .connect(validator2)
                    .approveStake(validator4.address, "signature5"),

                bridgeStorage
                    .connect(validator3)
                    .approveStake(validator4.address, "signature6"),
            ]);

            const beforeValidatorsCount = await bridgeStorage.validatorCount();

            expect(beforeValidatorsCount).to.be.eq(4n);

            // change status to false. Test for (2*validator)+1 / 3
            // incase of 3 validators, it requires 3 validators but
            // the threshold becomes 2/3 when validator count is
            // increased to 11.
            await Promise.all([
                bridgeStorage
                    .connect(validator1)
                    .changeValidatorStatus(validator2.address, false),
                bridgeStorage
                    .connect(validator3)
                    .changeValidatorStatus(validator2.address, false),
                bridgeStorage
                    .connect(validator4)
                    .changeValidatorStatus(validator2.address, false),
            ]);

            const afterValidatorCount = await bridgeStorage.validatorCount();

            expect(afterValidatorCount).to.be.eq(3n);

            // change status back to true. count should increase by one
            await Promise.all([
                bridgeStorage
                    .connect(validator1)
                    .changeValidatorStatus(validator2.address, true),
                bridgeStorage
                    .connect(validator3)
                    .changeValidatorStatus(validator2.address, true),
                bridgeStorage
                    .connect(validator4)
                    .changeValidatorStatus(validator2.address, true),
            ]);

            expect(await bridgeStorage.validatorCount()).to.be.eq(4n);
        });

        it("Should fail to vote if already voted", async function () {
            await bridgeStorage
                .connect(validator1)
                .approveStake(validator2.address, "signature1");

            await Promise.all([
                bridgeStorage
                    .connect(validator1)
                    .approveStake(validator3.address, "signature2"),

                bridgeStorage
                    .connect(validator2)
                    .approveStake(validator3.address, "signature3"),
            ]);

            await bridgeStorage
                .connect(validator1)
                .approveStake(validator4.address, "signature4");

            await expect(
                bridgeStorage
                    .connect(validator1)
                    .approveStake(validator4.address, "signature5")
            ).to.be.revertedWith("Already voted for this validator");
        });
    });

    describe("approveStake()", async function () {
        it("should approve stake and change validator status", async function () {
            const signature = "signature";
            const isValidatorBefore = await bridgeStorage.validators(
                validator2.address
            );
            await bridgeStorage
                .connect(validator1)
                .approveStake(validator2.address, signature);

            const isValidatorAfter = await bridgeStorage.validators(
                validator2.address
            );
            const signaturesCount =
                await bridgeStorage.getStakingSignaturesCount(
                    validator2.address
                );
            const validatorCount = await bridgeStorage.validatorCount();

            const [storedValidatorAddress, storedSignature] =
                await bridgeStorage
                    .connect(validator1)
                    .stakingSignatures(validator2.address, 0);

            expect(storedSignature).to.be.eq(signature);
            expect(storedValidatorAddress).to.be.eq(validator1.address);
            expect(isValidatorAfter).to.equal(true);
            expect(isValidatorBefore).to.equal(false);
            expect(signaturesCount).to.equal(1);
            expect(validatorCount).to.be.eq(2n);
        });

        it("should not approve stake with a used signature", async function () {
            await bridgeStorage
                .connect(validator1)
                .approveStake(owner.address, "signature");
            const signaturesCountBefore =
                await bridgeStorage.getStakingSignaturesCount(owner.address);

            // Attempt to approve stake with the same signature again
            await expect(
                bridgeStorage
                    .connect(validator1)
                    .approveStake(owner.address, "signature")
            ).to.be.revertedWith("Signature already used");

            const signaturesCountAfter =
                await bridgeStorage.getStakingSignaturesCount(owner.address);

            expect(signaturesCountBefore).to.equal(1);
            expect(signaturesCountAfter).to.equal(1); // Signature count should remain the same
        });

        it("should not approve stake if caller is not a validator", async function () {
            await expect(
                bridgeStorage
                    .connect(owner)
                    .approveStake(owner.address, "signature")
            ).to.be.rejectedWith("Only validators can call this function");
        });
    });

    describe("approveLockNft()", async function () {
        it("should fail if caller is not a validator", async function () {
            await expect(
                bridgeStorage
                    .connect(owner)
                    .approveLockNft(
                        "dummy_hash",
                        "dummy_chain",
                        "dummy_signature"
                    )
            ).to.be.revertedWith("Only validators can call this function");
        });

        it("should fail if signature is already used", async function () {
            await bridgeStorage
                .connect(validator1)
                .approveLockNft("dummy_hash", "dummy_chain", "dummy_signature")
                .then((r) => r.wait());

            await expect(
                bridgeStorage
                    .connect(validator1)
                    .approveLockNft(
                        "dummy_hash",
                        "dummy_chain",
                        "dummy_signature"
                    )
                    .then((r) => r.wait())
            ).to.be.revertedWith("Signature already used");
        });

        it("should successfully update the state", async function () {
            const hash = "dummy_hash";
            const chain = "dummy_chain";
            const signature = "dummy_signature";

            // no items initially so item at index
            const index = 0;

            await bridgeStorage
                .connect(validator1)
                .approveLockNft(hash, chain, signature)
                .then((r) => r.wait());

            const [validatorAddress, storedSignature] = await bridgeStorage
                .connect(validator1)
                .lockSignatures(hash, chain, index);

            expect(storedSignature).to.be.eq(signature);
            expect(validatorAddress).to.be.eq(validator1.address);
        });
    });
});
