// import assert from "assert";
// import { createHash } from "crypto";
// import * as fs from "fs";
// import * as dotenv from "dotenv";
// import * as ed from "@noble/ed25519";
// import * as anchor from "@project-serum/anchor";
// import { Program, BN, Spl } from "@project-serum/anchor";
// import {
//     Connection,
//     Ed25519Program,
//     Keypair,
//     LAMPORTS_PER_SOL,
//     PublicKey,
//     SystemProgram,
//     SYSVAR_INSTRUCTIONS_PUBKEY,
// } from "@solana/web3.js";
// import { serialize } from "@dao-xyz/borsh";

// import { XpBridge } from "../target/types/xp_bridge";
// import {
//     Collection,
//     Creator,
//     InitializeData,
//     PauseData,
//     TransferNftData,
//     UnfreezeNftData,
//     UnpauseData,
//     UpdateGroupkeyData,
//     WithdrawFeesData,
// } from "../src/encode";

// dotenv.config();

// const METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")

// describe("xp-bridge", () => {
//     // Configure the client to use the cluster provided by ANCHOR_PROVIDER_URL.
//     const url = process.env.ANCHOR_PROVIDER_URL;
//     const options = anchor.AnchorProvider.defaultOptions();
//     const connection = new Connection(url, options.commitment);
//     const payer = Keypair.fromSecretKey(
//         Buffer.from(
//             JSON.parse(
//                 fs.readFileSync(process.env.ANCHOR_WALLET, {
//                     encoding: "utf-8",
//                 })
//             )
//         )
//     );
//     const wallet = new anchor.Wallet(payer);
//     const provider = new anchor.AnchorProvider(connection, wallet, options);
//     anchor.setProvider(provider);

//     const tokenProgram = Spl.token();
//     const program = anchor.workspace.XpBridge as Program<XpBridge>;

//     let privateKey: Uint8Array;
//     let groupKey: Uint8Array;
//     let bridge: PublicKey;
//     let bridgeBump: number;

//     let validatorMapping: PublicKey;
//     let validatorBump: number;

//     let collectionMint: Keypair;
//     let tokenMint: Keypair;
//     let receiver: Keypair;
//     let _dummyMint: PublicKey;

//     let actionId: BN = new BN(0);

//     before(async () => {
//         privateKey = ed.utils.randomPrivateKey();
//         groupKey = await ed.getPublicKey(privateKey);

//         // const [authority] = await PublicKey.findProgramAddress(
//         //     [encode("auth")],
//         //     program.programId
//         // );

//         // collectionMint = Keypair.generate()
//         // await tokenProgram.methods
//         //     .initializeMint(0, authority, authority)
//         //     .accounts({
//         //         mint: collectionMint.publicKey,
//         //     })
//         //     .signers([collectionMint])
//         //     .preInstructions([
//         //         await tokenProgram.account.mint.createInstruction(collectionMint),
//         //     ])
//         //     .rpc();

//         // tokenMint = Keypair.generate();
//         // await tokenProgram.methods
//         //     .initializeMint(0, authority, authority)
//         //     .accounts({
//         //         mint: tokenMint.publicKey,
//         //     })
//         //     .signers([tokenMint])
//         //     .preInstructions([
//         //         await tokenProgram.account.mint.createInstruction(tokenMint),
//         //     ])
//         //     .rpc();

//         [bridge, bridgeBump] = await PublicKey.findProgramAddress(
//             [encode("bridge")],
//             program.programId
//         );

//         console.log(bridge, bridgeBump);


//         // [validatorMapping, validatorBump] = await PublicKey.findProgramAddress(
//         //     [
//         //         encode("validator"),
//         //     ],
//         //     program.programId
//         // );

//         // anchor.utils.bytes.utf8.encode('self-custodial-facebook2'),
//         //     userAddress.toBuffer(),

//         // receiver = Keypair.generate();
//         // const airdropSig = await provider.connection.requestAirdrop(
//         //     receiver.publicKey,
//         //     10 * LAMPORTS_PER_SOL
//         // );
//         // const latestBlockhash = await provider.connection.getLatestBlockhash();
//         // await provider.connection.confirmTransaction({
//         //     blockhash: latestBlockhash.blockhash,
//         //     lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
//         //     signature: airdropSig,
//         // });
//     });

//     it("initialize", async () => {

//         // const data = new InitializeData({
//         //     publicKey: wallet.publicKey
//         // })

//         // const message = serialize(data);
//         // const msgHash = createHash("SHA256").update(message).digest();
//         // const signature = await ed.sign(msgHash, privateKey);

//         // const ed25519Instruction =
//         //     Ed25519Program.createInstructionWithPublicKey({
//         //         publicKey: groupKey,
//         //         message: msgHash,
//         //         signature,
//         //     });



//         const tx = await program.methods
//             .initialize()
//             .accounts({
//                 bridge,
//                 // validatorsMapping: validatorMapping,
//                 user: provider.wallet.publicKey,
//                 systemProgram: SystemProgram.programId,
//             })
//             // .preInstructions([ed25519Instruction])
//             .rpc();

//         console.log("transaction signature", tx);

//         let bridgeAccount = await program.account.bridge.fetch(bridge);

//         console.log(bridgeAccount.validatorCount);

//         // const storedGroupKey = Buffer.from(bridgeAccount.groupKey);
//         // assert.ok(bridgeAccount.paused == false);
//         // assert.ok(storedGroupKey.equals(groupKey));
//     });

//     // it("pause", async () => {
//     //     actionId = actionId.add(new BN(1));

//     // const [consumedAction, _] = await PublicKey.findProgramAddress(
//     //     [actionId.toArrayLike(Buffer, "le", 8)],
//     //     program.programId
//     // );

//     //     const data = new PauseData({
//     //         actionId,
//     //         bridgeBump
//     //     });

//     //     const message = serialize(data);
//     //     const msgHash = createHash("SHA256").update(message).digest();
//     //     const signature = await ed.sign(msgHash, privateKey);

//     //     const ed25519Instruction =
//     //         Ed25519Program.createInstructionWithPublicKey({
//     //             publicKey: groupKey,
//     //             message: msgHash,
//     //             signature,
//     //         });

//     //     const tx = await program.methods
//     //         .validatePause(data)
//     //         .accounts({
//     //             bridge,
//     //             consumedAction,
//     //             user: provider.wallet.publicKey,
//     //             systemProgram: SystemProgram.programId,
//     //             instructionAcc: SYSVAR_INSTRUCTIONS_PUBKEY,
//     //         })
//     //         .preInstructions([ed25519Instruction])
//     //         .rpc();

//     //     console.log("transaction signature", tx);

//     //     let bridgeAccount = await program.account.bridge.fetch(bridge);

//     //     assert.ok(bridgeAccount.paused == true);
//     // });

//     // it("unpause", async () => {
//     //     actionId = actionId.add(new BN(1));
//     //     const [consumedAction, _] = await PublicKey.findProgramAddress(
//     //         [actionId.toArrayLike(Buffer, "le", 8)],
//     //         program.programId
//     //     );

//     //     const data = new UnpauseData({
//     //         actionId,
//     //         bridgeBump
//     //     });

//     //     const message = serialize(data);
//     //     const msgHash = createHash("SHA256").update(message).digest();
//     //     const signature = await ed.sign(msgHash, privateKey);

//     //     const ed25519Instruction =
//     //         Ed25519Program.createInstructionWithPublicKey({
//     //             publicKey: groupKey,
//     //             message: msgHash,
//     //             signature,
//     //         });
//     //     const tx = await program.methods
//     //         .validateUnpause(data)
//     //         .accounts({
//     //             bridge,
//     //             consumedAction,
//     //             user: provider.wallet.publicKey,
//     //             systemProgram: SystemProgram.programId,
//     //             instructionAcc: SYSVAR_INSTRUCTIONS_PUBKEY,
//     //         })
//     //         .preInstructions([ed25519Instruction])
//     //         .rpc();

//     //     console.log("transaction signature", tx);

//     //     let bridgeAccount = await program.account.bridge.fetch(bridge);

//     //     assert.ok(bridgeAccount.paused == false);
//     // });

//     // it("deploy a collection", async () => {
//     //     actionId = actionId.add(new BN(1));

//     //     const [consumedAction, _] = await PublicKey.findProgramAddress(
//     //         [actionId.toArrayLike(Buffer, "le", 8)],
//     //         program.programId
//     //     );

//     //     const [authority, authBump] = await PublicKey.findProgramAddress(
//     //         [encode("auth")],
//     //         program.programId
//     //     );

//     //     let tokenAccount = await getOrCreateTokenAccount(
//     //         collectionMint.publicKey,
//     //         receiver.publicKey
//     //     );

//     //     const [metadataAccount] = await PublicKey.findProgramAddress(
//     //         [
//     //             Buffer.from("metadata"),
//     //             METADATA_PROGRAM_ID.toBuffer(),
//     //             collectionMint.publicKey.toBuffer(),
//     //         ],
//     //         METADATA_PROGRAM_ID
//     //     );
//     //     const [editionAccount] = await PublicKey.findProgramAddress(
//     //         [
//     //             Buffer.from("metadata"),
//     //             METADATA_PROGRAM_ID.toBuffer(),
//     //             collectionMint.publicKey.toBuffer(),
//     //             Buffer.from("edition"),
//     //         ],
//     //         METADATA_PROGRAM_ID
//     //     );

//     //     const data = new TransferNftData({
//     //         actionId,
//     //         bridgeBump,
//     //         authBump,
//     //         chainNonce: new BN(0),
//     //         name: "Test",
//     //         symbol: "wNFT",
//     //         uri:
//     //             "https://v6ahotwazrvostarjcejqieltkiy5ireq7rwlqss4iezbgngakla.arweave.net/r4B3TsDMaulMEUiImCCLmpGOoiSH42XCUuIJkJmmApY/",
//     //         owner: receiver.publicKey,
//     //         collection: null,
//     //         sellerFeeBasisPoints: 0,
//     //         creators: [
//     //             new Creator({
//     //                 address: authority,
//     //                 verified: false,
//     //                 share: 100
//     //             }),
//     //             new Creator({
//     //                 address: receiver.publicKey,
//     //                 verified: false,
//     //                 share: 0
//     //             })
//     //         ]
//     //     });
//     //     const message = serialize(data);
//     //     const msgHash = createHash("SHA256").update(message).digest();
//     //     const signature = await ed.sign(msgHash, privateKey);
//     //     const verifyInstruction = Ed25519Program.createInstructionWithPublicKey(
//     //         {
//     //             publicKey: groupKey,
//     //             message: msgHash,
//     //             signature: signature,
//     //         }
//     //     );
//     //     const tx = await program.methods
//     //         .validateTransferNft(data)
//     //         .accounts({
//     //             bridge,
//     //             authority,
//     //             mint: tokenAccount.mint,
//     //             tokenAccount: tokenAccount.address,
//     //             metadataAccount,
//     //             editionAccount,
//     //             metadataProgram: METADATA_PROGRAM_ID,
//     //             tokenProgram: tokenProgram.programId,
//     //             instructionAcc: SYSVAR_INSTRUCTIONS_PUBKEY,
//     //             consumedAction,
//     //         })
//     //         .preInstructions([verifyInstruction])
//     //         .rpc();

//     //     console.log("transaction signature:", tx);

//     //     const mintAccount = await tokenProgram.account.mint.fetch(
//     //         collectionMint.publicKey
//     //     );

//     //     tokenAccount = await getOrCreateTokenAccount(
//     //         collectionMint.publicKey,
//     //         receiver.publicKey
//     //     );

//     //     assert.ok(mintAccount.decimals == 0);
//     //     assert.ok(mintAccount.supply.eq(new BN(1)));

//     //     assert.ok(tokenAccount.owner.equals(receiver.publicKey));
//     // });

//     // it("validate_transfer_nft", async () => {
//     //     actionId = actionId.add(new BN(1));

//     //     const [consumedAction, _] = await PublicKey.findProgramAddress(
//     //         [actionId.toArrayLike(Buffer, "le", 8)],
//     //         program.programId
//     //     );

//     //     const [authority, authBump] = await PublicKey.findProgramAddress(
//     //         [encode("auth")],
//     //         program.programId
//     //     );

//     //     let tokenAccount = await getOrCreateTokenAccount(
//     //         tokenMint.publicKey,
//     //         receiver.publicKey
//     //     );

//     //     const [metadataAccount] = await PublicKey.findProgramAddress(
//     //         [
//     //             Buffer.from("metadata"),
//     //             METADATA_PROGRAM_ID.toBuffer(),
//     //             tokenMint.publicKey.toBuffer(),
//     //         ],
//     //         METADATA_PROGRAM_ID
//     //     );
//     //     const [editionAccount] = await PublicKey.findProgramAddress(
//     //         [
//     //             Buffer.from("metadata"),
//     //             METADATA_PROGRAM_ID.toBuffer(),
//     //             tokenMint.publicKey.toBuffer(),
//     //             Buffer.from("edition"),
//     //         ],
//     //         METADATA_PROGRAM_ID
//     //     );

//     //     const data = new TransferNftData({
//     //         actionId,
//     //         bridgeBump,
//     //         authBump,
//     //         chainNonce: new BN(0),
//     //         name: "Test",
//     //         symbol: "wNFT",
//     //         uri:
//     //             "https://v6ahotwazrvostarjcejqieltkiy5ireq7rwlqss4iezbgngakla.arweave.net/r4B3TsDMaulMEUiImCCLmpGOoiSH42XCUuIJkJmmApY/",
//     //         owner: receiver.publicKey,
//     //         collection: new Collection({
//     //             verified: false,
//     //             key: collectionMint.publicKey,
//     //         }),
//     //         sellerFeeBasisPoints: null,
//     //         creators: null
//     //     });
//     //     const message = serialize(data);
//     //     const msgHash = createHash("SHA256").update(message).digest();
//     //     const signature = await ed.sign(msgHash, privateKey);
//     //     const verifyInstruction = Ed25519Program.createInstructionWithPublicKey(
//     //         {
//     //             publicKey: groupKey,
//     //             message: msgHash,
//     //             signature: signature,
//     //         }
//     //     );

//     //     try {
//     //         const tx = await program.methods
//     //             .validateTransferNft(data)
//     //             .accounts({
//     //                 bridge,
//     //                 authority,
//     //                 mint: tokenAccount.mint,
//     //                 tokenAccount: tokenAccount.address,
//     //                 metadataAccount,
//     //                 editionAccount,
//     //                 metadataProgram: METADATA_PROGRAM_ID,
//     //                 tokenProgram: tokenProgram.programId,
//     //                 instructionAcc: SYSVAR_INSTRUCTIONS_PUBKEY,
//     //                 consumedAction,
//     //             })
//     //             .preInstructions([verifyInstruction])
//     //             .rpc();

//     //         console.log("transaction signature:", tx);
//     //     } catch(e) {
//     //         console.log(e)
//     //     }

//     //     const mintAccount = await tokenProgram.account.mint.fetch(
//     //         tokenMint.publicKey
//     //     );

//     //     tokenAccount = await getOrCreateTokenAccount(
//     //         tokenMint.publicKey,
//     //         receiver.publicKey
//     //     );

//     //     assert.ok(mintAccount.decimals == 0);
//     //     assert.ok(mintAccount.supply.eq(new BN(1)));

//     //     assert.ok(tokenAccount.owner.equals(receiver.publicKey));
//     // });

//     // it("withdraw nft", async () => {
//     //     const chainNonce = 0;
//     //     const targetAddress =
//     //         "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
//     //     const lamports = new BN(LAMPORTS_PER_SOL / 100);

//     //     let tokenAccount = await getOrCreateTokenAccount(
//     //         tokenMint.publicKey,
//     //         receiver.publicKey
//     //     );

//     //     let listener = null;

//     //     let [event, slot] = await new Promise(async (resolve, _reject) => {
//     //         listener = program.addEventListener(
//     //             "UnfreezeNft",
//     //             (event, slot) => {
//     //                 resolve([event, slot]);
//     //             }
//     //         );
//     //         const tx = await program.methods
//     //             .withdrawNft(chainNonce, targetAddress, lamports, bridgeBump)
//     //             .accounts({
//     //                 bridge,
//     //                 authority: receiver.publicKey,
//     //                 mint: tokenAccount.mint,
//     //                 tokenAccount: tokenAccount.address,
//     //                 tokenProgram: tokenProgram.programId,
//     //             })
//     //             .signers([receiver])
//     //             .rpc();

//     //         console.log("transaction signature", tx);
//     //     });
//     //     await program.removeEventListener(listener);

//     //     assert.ok(slot > 0);

//     //     const mintInfo = await tokenProgram.account.mint.fetch(
//     //         tokenMint.publicKey
//     //     );
//     //     tokenAccount = await getOrCreateTokenAccount(
//     //         tokenMint.publicKey,
//     //         receiver.publicKey
//     //     );

//     //     assert.ok(tokenAccount.amount == BigInt(0));
//     //     assert.ok(mintInfo.supply.eq(new BN(0)));
//     //     assert.ok(event.to == targetAddress);
//     //     assert.ok(event.chainNonce == chainNonce);
//     //     assert.ok(event.mint.equals(tokenMint.publicKey));
//     // });

//     // it("freeze nft", async () => {
//     //     const sender = Keypair.generate();
//     //     const dummyMint = await createDummyNft(provider.connection, sender);

//     //     let fromAccount = await getOrCreateTokenAccount(
//     //         dummyMint,
//     //         sender.publicKey
//     //     );
//     //     let toAccount = await getOrCreateTokenAccount(dummyMint, bridge);

//     //     const chainNonce = 0;
//     //     const targetAddress =
//     //         "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"; // address on other chain
//     //     const lamports = new BN(LAMPORTS_PER_SOL / 100);
//     //     const mintWith = "mintWith";

//     //     let listener = null;
//     //     let [event, slot] = await new Promise(async (resolve, _reject) => {
//     //         listener = program.addEventListener(
//     //             "TransferNft",
//     //             (event, slot) => {
//     //                 resolve([event, slot]);
//     //             }
//     //         );
//     //         const tx = await program.methods
//     //             .freezeNft(chainNonce, targetAddress, lamports, mintWith, bridgeBump)
//     //             .accounts({
//     //                 bridge,
//     //                 authority: sender.publicKey,
//     //                 from: fromAccount.address,
//     //                 to: toAccount.address,
//     //                 tokenProgram: tokenProgram.programId,
//     //             })
//     //             .signers([sender])
//     //             .rpc();
//     //         console.log("transaction signature", tx);
//     //     });
//     //     await program.removeEventListener(listener);

//     //     assert.ok(slot > 0);

//     //     fromAccount = await getOrCreateTokenAccount(
//     //         dummyMint,
//     //         sender.publicKey
//     //     );
//     //     toAccount = await getOrCreateTokenAccount(dummyMint, bridge);

//     //     assert.ok(fromAccount.amount == BigInt(0));
//     //     assert.ok(toAccount.amount == BigInt(1));
//     //     assert.ok(toAccount.owner.equals(bridge));
//     //     assert.ok(event.to == targetAddress);
//     //     assert.ok(event.chainNonce == chainNonce);
//     //     assert.ok(event.mint.equals(dummyMint));
//     //     assert.ok(event.mintWith == mintWith);

//     //     _dummyMint = dummyMint;
//     // });

//     // it("validate unfreeze nft", async () => {
//     //     actionId = actionId.add(new BN(1));

//     //     const [consumedAction, _] = await PublicKey.findProgramAddress(
//     //         [actionId.toArrayLike(Buffer, "le", 8)],
//     //         program.programId
//     //     );

//     //     const dummyMint = _dummyMint;
//     //     const receiver = Keypair.generate();

//     //     let bridgeTokenAccount = await getOrCreateTokenAccount(
//     //         dummyMint,
//     //         bridge
//     //     );
//     //     let receiverTokenAccount = await getOrCreateTokenAccount(
//     //         dummyMint,
//     //         receiver.publicKey
//     //     );

//     //     const data = new UnfreezeNftData({
//     //         actionId,
//     //         bridgeBump,
//     //         receiver: receiver.publicKey,
//     //         mint: dummyMint,
//     //     });
//     //     const message = serialize(data);
//     //     const msgHash = createHash("SHA256").update(message).digest();
//     //     const signature = await ed.sign(msgHash, privateKey);
//     //     const verifyInstruction = Ed25519Program.createInstructionWithPublicKey(
//     //         {
//     //             publicKey: groupKey,
//     //             message: msgHash,
//     //             signature: signature,
//     //         }
//     //     );
//     //     const tx = await program.methods
//     //         .validateUnfreezeNft(data)
//     //         .accounts({
//     //             bridge,
//     //             from: bridgeTokenAccount.address,
//     //             to: receiverTokenAccount.address,
//     //             tokenProgram: tokenProgram.programId,
//     //             consumedAction,
//     //             payer: provider.wallet.publicKey,
//     //             systemProgram: SystemProgram.programId,
//     //             instructionAcc: SYSVAR_INSTRUCTIONS_PUBKEY,
//     //         })
//     //         .preInstructions([verifyInstruction])
//     //         .rpc();

//     //     console.log("transaction signature", tx);

//     //     bridgeTokenAccount = await getOrCreateTokenAccount(dummyMint, bridge);
//     //     receiverTokenAccount = await getOrCreateTokenAccount(
//     //         dummyMint,
//     //         receiver.publicKey
//     //     );

//     //     assert.ok(bridgeTokenAccount.amount == BigInt(0));
//     //     assert.ok(receiverTokenAccount.amount == BigInt(1));
//     // });

//     // it("withdraw fees", async () => {
//     //     actionId = actionId.add(new BN(1));

//     //     const [consumedAction, _] = await PublicKey.findProgramAddress(
//     //         [actionId.toArrayLike(Buffer, "le", 8)],
//     //         program.programId
//     //     );

//     //     const data = new WithdrawFeesData({
//     //         actionId,
//     //         bridgeBump
//     //     });

//     //     const message = serialize(data);
//     //     const msgHash = createHash("SHA256").update(message).digest();
//     //     const signature = await ed.sign(msgHash, privateKey);

//     //     const verifyInstruction = Ed25519Program.createInstructionWithPublicKey(
//     //         {
//     //             publicKey: groupKey,
//     //             message: msgHash,
//     //             signature,
//     //         }
//     //     );
//     //     const tx = await program.methods
//     //         .validateWithdrawFees(data)
//     //         .accounts({
//     //             bridge,
//     //             consumedAction,
//     //             instructionAcc: SYSVAR_INSTRUCTIONS_PUBKEY,
//     //         })
//     //         .preInstructions([verifyInstruction])
//     //         .rpc();
//     //     console.log("transaction signature", tx);
//     // });

//     // it("update group key", async () => {
//     //     actionId = actionId.add(new BN(1));

//     //     const [consumedAction, _] = await PublicKey.findProgramAddress(
//     //         [actionId.toArrayLike(Buffer, "le", 8)],
//     //         program.programId
//     //     );

//     //     const newPrivateKey = ed.utils.randomPrivateKey();
//     //     const newKey = await ed.getPublicKey(newPrivateKey);
//     //     const data = new UpdateGroupkeyData({
//     //         actionId,
//     //         bridgeBump,
//     //         newKey: [...newKey],
//     //     });

//     //     const message = serialize(data);
//     //     const msgHash = createHash("SHA256").update(message).digest();
//     //     const signature = await ed.sign(msgHash, privateKey);

//     //     const ed25519Instruction =
//     //         Ed25519Program.createInstructionWithPublicKey({
//     //             publicKey: groupKey,
//     //             message: msgHash,
//     //             signature,
//     //         });
//     //     const tx = await program.methods
//     //         .validateUpdateGroupkey(data)
//     //         .accounts({
//     //             bridge,
//     //             consumedAction,
//     //             user: provider.wallet.publicKey,
//     //             systemProgram: SystemProgram.programId,
//     //             instructionAcc: SYSVAR_INSTRUCTIONS_PUBKEY,
//     //         })
//     //         .preInstructions([ed25519Instruction])
//     //         .rpc();

//     //     console.log("transaction signature", tx);

//     //     let bridgeAccount = await program.account.bridge.fetch(bridge);

//     //     const storedGroupKey = Buffer.from(bridgeAccount.groupKey);
//     //     assert.ok(storedGroupKey.equals(newKey));
//     // });
// });

// async function createDummyNft(connection: Connection, user: Keypair) {
//     const sig = await connection.requestAirdrop(
//         user.publicKey,
//         10 * LAMPORTS_PER_SOL
//     );

//     const latestBlockHash = await connection.getLatestBlockhash();

//     await connection.confirmTransaction({
//         blockhash: latestBlockHash.blockhash,
//         lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
//         signature: sig,
//     });

//     const splProgram = Spl.token();
//     const mint = Keypair.generate();
//     await splProgram.methods
//         .initializeMint(0, user.publicKey, user.publicKey)
//         .accounts({
//             mint: mint.publicKey,
//         })
//         .signers([mint])
//         .preInstructions([
//             await splProgram.account.mint.createInstruction(mint),
//         ])
//         .rpc();

//     const userAssociatedAccount = await getOrCreateTokenAccount(
//         mint.publicKey,
//         user.publicKey
//     );

//     await splProgram.methods
//         .mintTo(new BN(1))
//         .accounts({
//             mint: mint.publicKey,
//             to: userAssociatedAccount.address,
//             authority: user.publicKey,
//         })
//         .signers([user])
//         .rpc();

//     await splProgram.methods
//         .setAuthority(0 /* = AuthorityType.MintTokens */, undefined)
//         .accounts({
//             mint: mint.publicKey,
//             authority: user.publicKey,
//         })
//         .signers([user])
//         .rpc();

//     // let metadataData = new MetadataDataData({
//     //     name: "Test",
//     //     symbol: "FOO",
//     //     uri: "https://v6ahotwazrvostarjcejqieltkiy5ireq7rwlqss4iezbgngakla.arweave.net/r4B3TsDMaulMEUiImCCLmpGOoiSH42XCUuIJkJmmApY/",
//     //     sellerFeeBasisPoints: 500,
//     //     creators: null,
//     // });

//     // const res = await actions.createMetadata({
//     //     connection,
//     //     wallet: new NodeWallet(user),
//     //     metadataData,
//     //     editionMint: mintAccount.publicKey,
//     // });
//     return mint.publicKey;
// }

// async function getOrCreateTokenAccount(mint: PublicKey, owner: PublicKey) {
//     const tokenProgram = Spl.token();
//     const program = Spl.associatedToken();

//     const [associatedToken] = await PublicKey.findProgramAddress(
//         [owner.toBuffer(), tokenProgram.programId.toBuffer(), mint.toBuffer()],
//         program.programId
//     );

//     try {
//         const tokenAccount = await tokenProgram.account.token.fetch(
//             associatedToken
//         );
//         return {
//             address: associatedToken,
//             owner: tokenAccount.authority,
//             ...tokenAccount,
//         };
//     } catch (e) {
//         try {
//             await program.methods
//                 .create()
//                 .accounts({
//                     mint,
//                     owner,
//                     associatedAccount: associatedToken,
//                 })
//                 .rpc();

//             const tokenAccount = await tokenProgram.account.token.fetch(
//                 associatedToken
//             );
//             return {
//                 address: associatedToken,
//                 owner: tokenAccount.authority,
//                 ...tokenAccount,
//             };
//         } catch (e) {
//             throw e;
//         }
//     }
// }
import * as anchor from "@project-serum/anchor";
import { BN, Program, Spl } from "@project-serum/anchor";
import { XpBridge } from "../target/types/xp_bridge";
import { Connection, Ed25519Program, Keypair, PublicKey, SYSVAR_INSTRUCTIONS_PUBKEY, SystemProgram, Transaction } from "@solana/web3.js";
import * as fs from "fs";
import { AddValidatorData, ClaimData, ClaimNftData, InitializeData, LockData, NewValidatorPublicKey, SignatureInfo, TxHash, VerifyAddValidatorSignaturesData, VerifyClaimSignaturesData } from "../src/encode";
import { createHash } from "crypto";
import { serialize } from "@dao-xyz/borsh";
import * as ed from "@noble/ed25519";
import { Metaplex } from "@metaplex-foundation/js";
import * as spl from "@solana/spl-token";
import {
  Metadata,
  PROGRAM_ID as METADATA_PROGRAM_ID,
} from "@metaplex-foundation/mpl-token-metadata";
import { TOKEN_PROGRAM_ID } from "@project-serum/anchor/dist/cjs/utils/token";


/**
 * @param tx a solana transaction
 * @param feePayer the publicKey of the signer
 * @returns size in bytes of the transaction
 */
const getTxSize = (tx: Transaction, feePayer: PublicKey): number => {
  const feePayerPk = [feePayer.toBase58()];

  const signers = new Set<string>(feePayerPk);
  const accounts = new Set<string>(feePayerPk);

  const ixsSize = tx.instructions.reduce((acc, ix) => {
    ix.keys.forEach(({ pubkey, isSigner }) => {
      const pk = pubkey.toBase58();
      if (isSigner) signers.add(pk);
      accounts.add(pk);
    });

    accounts.add(ix.programId.toBase58());

    const nIndexes = ix.keys.length;
    const opaqueData = ix.data.length;

    return (
      acc +
      1 + // PID index
      compactArraySize(nIndexes, 1) +
      compactArraySize(opaqueData, 1)
    );
  }, 0);

  return (
    compactArraySize(signers.size, 64) + // signatures
    3 + // header
    compactArraySize(accounts.size, 32) + // accounts
    32 + // blockhash
    compactHeader(tx.instructions.length) + // instructions
    ixsSize
  );
};

// COMPACT ARRAY

const LOW_VALUE = 127; // 0x7f
const HIGH_VALUE = 16383; // 0x3fff

/**
* Compact u16 array header size
* @param n elements in the compact array
* @returns size in bytes of array header
*/
const compactHeader = (n: number) => (n <= LOW_VALUE ? 1 : n <= HIGH_VALUE ? 2 : 3);

/**
* Compact u16 array size
* @param n elements in the compact array
* @param size bytes per each element
* @returns size in bytes of array
*/
const compactArraySize = (n: number, size: number) => compactHeader(n) + n * size;

describe("bridge", async () => {
  const url = process.env.ANCHOR_PROVIDER_URL;
  const options = anchor.AnchorProvider.defaultOptions();
  const connection = new Connection(url, options.commitment);
  const payer = Keypair.fromSecretKey(
    Buffer.from(
      JSON.parse(
        fs.readFileSync(process.env.ANCHOR_WALLET, {
          encoding: "utf-8",
        })
      )
    )
  );
  const wallet = new anchor.Wallet(payer);
  const provider = new anchor.AnchorProvider(connection, wallet, options);
  anchor.setProvider(provider);
  const encode = anchor.utils.bytes.utf8.encode;
  const program = anchor.workspace.XpBridge as Program<XpBridge>;
  const metaplex = Metaplex.make(connection);

  const SELF_CHAIN = "SOL";
  const TYPE_NFT = "nft"; // a more general term to accomodate non-evm chains
  const TYPE_SFT = "sft"; // a more general term to accomodate non-evm chains
  const BRIDGE = "bridge";
  const VALIDATOR = "validator";
  const THRESHOLD = "threshold";
  const SELF_TOKENS = "self_tokens";
  const OTHER_TOKENS = "other_tokens";
  const ORIGINAL_TO_DUPLICATE_MAPPING = "original_to_duplicate_mapping";
  const DUPLICATE_TO_ORIGINAL_MAPPING = "duplicate_to_original_mapping";

  //Bridge account
  let bridge: PublicKey;
  let bridgeBump: number;

  //Validators account
  let validator: PublicKey;
  let validatorBump: number;

  //Other tokens account
  let otherTokens: PublicKey;
  let otherTokensBump: number;

  //Self tokens account
  let selfTokens: PublicKey;
  let selfTokensBump: number;

  //Original to duplicate account
  let otdm: PublicKey;
  let otdmBump: number;

  //Duplicate to original account
  let dtom: PublicKey;
  let dtomBump: number;

  //Threshold account
  let threshold: PublicKey;
  let thresholdBump: number;

  [bridge, bridgeBump] = await PublicKey.findProgramAddress(
    [encode(BRIDGE)],
    program.programId
  );

  [validator, validatorBump] = await PublicKey.findProgramAddress(
    [new Uint8Array(createHash("SHA256").update(Buffer.from(`${VALIDATOR}${wallet.publicKey.toString()}`)).digest())],
    program.programId
  );

  // it("Initialization", async () => {

  //   const data = new InitializeData({
  //     validatorPublicKey: wallet.publicKey
  //   });

  //   try {
  //     //@ts-ignore
  //     const tx = await program.methods.initialize(data)
  //       .accounts({
  //         bridge: bridge,
  //         validators: validator,
  //         user: provider.wallet.publicKey,
  //         systemProgram: SystemProgram.programId,
  //       })
  //       .rpc({
  //         skipPreflight: false
  //       });

  //     console.log("TxHash ::", tx);
  //   }
  //   catch (ex) {
  //     console.log(ex);
  //   }
  //   let bridgeAccount = await program.account.bridge.fetch(bridge);

  //   console.log(bridgeAccount.validatorCount);

  // });

  // it("Verify Add Validator Sig And Add Validator", async () => {
  //   let newValidatorKeys = Keypair.generate();

  //   let newValidator = new NewValidatorPublicKey({
  //     validatorPublicKey: newValidatorKeys.publicKey
  //   });

  //   const message = serialize(newValidator);
  //   const msgHash = createHash("SHA256").update(message).digest();
  //   const signature = await ed.sign(msgHash, wallet.payer.secretKey.slice(0, 32));

  //   const ed25519Instruction =
  //     Ed25519Program.createInstructionWithPublicKey({
  //       publicKey: wallet.payer.publicKey.toBuffer(),
  //       message: msgHash,
  //       signature,
  //     });

  //   const signatures = [new SignatureInfo({
  //     publicKey: wallet.publicKey,
  //     sig: Array.from(signature)
  //   }), new SignatureInfo({
  //     publicKey: wallet.publicKey,
  //     sig: Array.from(signature)
  //   }), new SignatureInfo({
  //     publicKey: wallet.publicKey,
  //     sig: Array.from(signature)
  //   })];

  //   console.log(signatures.length);

  //   const data = new VerifyAddValidatorSignaturesData({
  //     validatorPublicKey: newValidatorKeys.publicKey,
  //     signatures: signatures
  //   });

  //   [threshold, thresholdBump] = await PublicKey.findProgramAddress(
  //     [new Uint8Array(createHash("SHA256").update(Buffer.from(`${THRESHOLD}${newValidatorKeys.publicKey.toString()}`)).digest())],
  //     program.programId
  //   )

  //   try {
  //     // const modifyComputeUnits =
  //     //   anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
  //     //     units: 300_000,
  //     //   });

  //     const tx = await program.methods.verifyAddValidatorSignatures(data)
  //       .accounts({
  //         threshold: threshold,
  //         user: provider.wallet.publicKey,
  //         systemProgram: SystemProgram.programId,
  //         instructionAcc: SYSVAR_INSTRUCTIONS_PUBKEY
  //       })
  //       .remainingAccounts(
  //         [
  //           { isSigner: false, isWritable: false, pubkey: validator },
  //           { isSigner: false, isWritable: false, pubkey: validator },
  //           { isSigner: false, isWritable: false, pubkey: validator }
  //         ]
  //       )
  //       .preInstructions(
  //         [
  //           ed25519Instruction,
  //           ed25519Instruction,
  //           ed25519Instruction
  //         ]
  //       )
  //       .transaction()

  //     // const transferTransaction = new anchor.web3.Transaction().add(
  //     //   modifyComputeUnits,
  //     //   tx
  //     // );

  //     let size = getTxSize(tx, wallet.payer.publicKey);
  //     console.log("size ::", size);

  //     const txSig = await anchor.web3.sendAndConfirmTransaction(
  //       connection,
  //       tx,
  //       [wallet.payer],
  //       { skipPreflight: false }
  //     );
  //     console.log("TxHash ::", txSig);
  //   }
  //   catch (ex) {
  //     console.log(ex);
  //   }

  //   const addValidatordata = new AddValidatorData({
  //     validatorPublicKey: newValidatorKeys.publicKey,
  //   });

  //   [validator, validatorBump] = await PublicKey.findProgramAddress(
  //     [new Uint8Array(createHash("SHA256").update(Buffer.from(`${VALIDATOR}${newValidatorKeys.publicKey.toString()}`)).digest())],
  //     program.programId
  //   );

  //   [threshold, thresholdBump] = await PublicKey.findProgramAddress(
  //     [new Uint8Array(createHash("SHA256").update(Buffer.from(`${THRESHOLD}${newValidatorKeys.publicKey.toString()}`)).digest())],
  //     program.programId
  //   );

  //   try {
  //     //@ts-ignore
  //     const tx = await program.methods.addValidator(addValidatordata)
  //       .accounts({
  //         bridge: bridge,
  //         validators: validator,
  //         //@ts-ignore
  //         threshold: threshold,
  //         user: provider.wallet.publicKey,
  //         systemProgram: SystemProgram.programId,
  //         instructionAcc: SYSVAR_INSTRUCTIONS_PUBKEY
  //       })
  //       .preInstructions([ed25519Instruction, ed25519Instruction, ed25519Instruction, ed25519Instruction])
  //       .transaction()


  //     let size = getTxSize(tx, wallet.payer.publicKey);
  //     console.log("size ::", size);

  //     const txSig = await anchor.web3.sendAndConfirmTransaction(
  //       connection,
  //       tx,
  //       [wallet.payer],
  //       { skipPreflight: false }
  //     );
  //     console.log("TxHash ::", txSig);
  //   }
  //   catch (ex) {
  //     console.log(ex);
  //   }

  // });

  // it("Verify Claim Sig", async () => {

  //   let claimData = new ClaimNftData({
  //     tokenId: "1",
  //     sourceChain: "BSC",
  //     destinationChain: "SOL",
  //     destinationUserAddress: wallet.publicKey,
  //     sourceNftContractAddress: "0x4a27453D9D811AF3e0D7E574279aAD2569bA5b28",
  //     name: "name",
  //     symbol: "symbol",
  //     royalty: new BN(0),
  //     royaltyReceiver: wallet.publicKey,
  //     metadata: "https://bafkreianwnedmty7tbdgr5rc6udztk2rm2zufr7hsaqwqb6wwijtxhsksu.ipfs.nftstorage.link",
  //     transactionHash: "0xc237d45d8ddaddd4721e111cde64a1fec6717c651c27376563cdff5ed02d7553",
  //     tokenAmount: new BN(1),
  //     nftType: "nft",
  //     fee: new BN(0),
  //   });

  //   let txHashArray = Array.from(createHash("SHA256").update(claimData.transactionHash).digest());

  //   let txHash = new TxHash({
  //     txHash: txHashArray
  //   });

  //   const message = serialize(claimData);
  //   const msgHash = createHash("SHA256").update(message).digest();
  //   const signature = await ed.sign(msgHash, wallet.payer.secretKey.slice(0, 32));

  //   const ed25519Instruction =
  //     Ed25519Program.createInstructionWithPublicKey({
  //       publicKey: wallet.payer.publicKey.toBuffer(),
  //       message: msgHash,
  //       signature,
  //     });

  //   const signatures = [new SignatureInfo({
  //     publicKey: wallet.publicKey,
  //     sig: Array.from(signature)
  //   }), new SignatureInfo({
  //     publicKey: wallet.publicKey,
  //     sig: Array.from(signature)
  //   })];


  //   const data = new VerifyClaimSignaturesData({
  //     txHash: txHashArray,
  //     claimData,
  //     signatures: signatures
  //   });

  //   [threshold, thresholdBump] = await PublicKey.findProgramAddress(
  //     [new Uint8Array(createHash("SHA256").update(serialize(txHash)).digest())],
  //     program.programId
  //   )

  //   try {
  //     // const modifyComputeUnits =
  //     //   anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
  //     //     units: 300_000,
  //     //   });
  //     //@ts-ignore
  //     const tx = await program.methods.verifyClaimSignatures(data)
  //       .accounts({
  //         threshold: threshold,
  //         user: provider.wallet.publicKey,
  //         systemProgram: SystemProgram.programId,
  //         instructionAcc: SYSVAR_INSTRUCTIONS_PUBKEY
  //       })
  //       .remainingAccounts(
  //         [
  //           { isSigner: false, isWritable: false, pubkey: validator },
  //           { isSigner: false, isWritable: false, pubkey: validator },
  //         ]
  //       )
  //       .preInstructions(
  //         [
  //           ed25519Instruction,
  //           ed25519Instruction,
  //         ]
  //       )
  //       .transaction()

  //     // const transferTransaction = new anchor.web3.Transaction().add(
  //     //   modifyComputeUnits,
  //     //   tx
  //     // );

  //     let size = getTxSize(tx, wallet.payer.publicKey);
  //     console.log("size ::", size);

  //     const txSig = await anchor.web3.sendAndConfirmTransaction(
  //       connection,
  //       tx,
  //       [wallet.payer],
  //       { skipPreflight: false }
  //     );
  //     console.log("TxHash ::", txSig);
  //   }
  //   catch (ex) {
  //     console.log(ex);
  //   }
  // });

  // it("Lock721", async () => {

  //   const nftKey = new anchor.web3.PublicKey("DBXskGEvW8jkdCaBfrqLmMUQPV1wMGJZTMQiVWBcBvt5");
  //   const collectionKey = new anchor.web3.PublicKey("4WC7NEPXNeLnvcPX5NfB3YWRq9ez8PDsKBzDefYLrv9m");

  //   const fromTokenAccount = await spl.getAssociatedTokenAddress(
  //     nftKey,
  //     wallet.publicKey
  //   );

  //   console.log("from token account", fromTokenAccount.toString());

  //   const toTokenAccount = await getOrCreateTokenAccount(nftKey, bridge);

  //   console.log("to token account", toTokenAccount.address.toString());

  //   [otherTokens, otherTokensBump] = await PublicKey.findProgramAddress(
  //     [new Uint8Array(createHash("SHA256").update(Buffer.from(`other_tokens${nftKey.toString()}SOL${collectionKey.toString()}`)).digest())],
  //     program.programId
  //   );

  //   [selfTokens, selfTokensBump] = await PublicKey.findProgramAddress(
  //     [new Uint8Array(createHash("SHA256").update(Buffer.from(`self_tokens${nftKey.toString()}SOL${collectionKey.toString()}`)).digest())],
  //     program.programId
  //   );

  //   [dtom, dtomBump] = await PublicKey.findProgramAddress(
  //     [new Uint8Array(createHash("SHA256").update(Buffer.from(`duplicate_to_original_mapping${collectionKey.toString()}SOL`)).digest())],
  //     program.programId
  //   );

  //   // try {
  //   //   let isOtherTokensExists = await program.account.otherTokenInfo.fetch(otherTokens);
  //   //   console.log("isOtherTokensExists", isOtherTokensExists);
  //   //   [selfTokens, selfTokensBump] = await PublicKey.findProgramAddress(
  //   //     [new Uint8Array(createHash("SHA256").update(Buffer.from(`self_tokens${isOtherTokensExists.tokenId.toString()}${isOtherTokensExists.chain}${isOtherTokensExists.contractAddress}`)).digest())],
  //   //     program.programId
  //   //   );
  //   // }
  //   // catch (ex) {
  //   //   console.log(ex);
  //   // }

  //   // console.log("OTHER", await program.account.otherTokenInfo.fetch(otherTokens));
  //   // console.log("SELF", await program.account.selfTokenInfo.fetch(selfTokens));

  //   let data = new LockData({
  //     tokenId: nftKey,
  //     destinationChain: "BSC",
  //     destinationUserAddress: "0x12344353",
  //     sourceNftContractAddress: collectionKey,
  //     tokenAmount: new BN(1),
  //     bridgeBump: bridgeBump,
  //     otherTokensBump: otherTokensBump,
  //     selfTokensBump: selfTokensBump,
  //   });

  //   try {
  //     const modifyComputeUnits =
  //       anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
  //         units: 300_000,
  //       });

  //     const tx = await program.methods.lockNft(data)
  //       .accounts({
  //         bridge: bridge,
  //         otherTokens: otherTokens,
  //         selfTokens: selfTokens,
  //         duplicateToOriginalMapping: dtom,
  //         authority: wallet.payer.publicKey,
  //         from: fromTokenAccount,
  //         to: toTokenAccount.address,
  //         systemProgram: SystemProgram.programId,
  //         tokenProgram: TOKEN_PROGRAM_ID
  //       })
  //       .transaction();

  //     const transferTransaction = new anchor.web3.Transaction().add(
  //       modifyComputeUnits,
  //       tx
  //     );
  //     const txSig = await anchor.web3.sendAndConfirmTransaction(
  //       connection,
  //       transferTransaction,
  //       [wallet.payer],
  //       { skipPreflight: false }
  //     );
  //     console.log("tx ::", txSig);

  //   }
  //   catch (ex) {
  //     console.log(ex);
  //   }
  // });
// return
  it("Claim721", async () => {

    let isCollectionExists = false;
    let isUnlock = false;

    let claimData = new ClaimNftData({
      tokenId: "8",
      sourceChain: "BSC",
      destinationChain: "SOL",
      destinationUserAddress: wallet.publicKey,
      sourceNftContractAddress: "0x4a27453Dvv811AF3e0D7E574279aAD2569bA5b28",
      name: "name",
      symbol: "symbol",
      royalty: new BN(0),
      royaltyReceiver: wallet.publicKey,
      metadata: "https://bafkreianwnedmty7tbdgr5rc6udztk2rm2zufr7hsaqwqb6wwijtxhsksu.ipfs.nftstorage.link",
      transactionHash: "0xc237d45d8ddaddd4721e11llpe64a1ss8l717c651c27376563cdff5ed02d7553",
      tokenAmount: new BN(1),
      nftType: "nft",
      fee: new BN(0),
    })

    const message = serialize(claimData);
    const msgHash = createHash("SHA256").update(message).digest();
    const signature = await ed.sign(msgHash, wallet.payer.secretKey.slice(0, 32));

    const signatures = [new SignatureInfo({
      publicKey: wallet.publicKey,
      sig: Array.from(signature)
    })];




    [selfTokens, selfTokensBump] = await PublicKey.findProgramAddress(
      [new Uint8Array(createHash("SHA256").update(Buffer.from(`self_tokens${claimData.tokenId}${claimData.sourceChain}${claimData.sourceNftContractAddress}`)).digest())],
      program.programId
    );



    [otdm, otdmBump] = await PublicKey.findProgramAddress(
      [new Uint8Array(createHash("SHA256").update(Buffer.from(`original_to_duplicate_mapping${claimData.sourceNftContractAddress}${claimData.sourceChain}`)).digest())],
      program.programId
    );

    let otdmData: any;
    let selfTokensData: any;
    let collectionPDA: anchor.web3.PublicKey;
    let collectionMetadataPDA: anchor.web3.PublicKey;
    let collectionMasterEditionPDA: anchor.web3.PublicKey;
    let collectionTokenAccount: anchor.web3.PublicKey;
    let mint: anchor.web3.PublicKey;
    let newMint: anchor.web3.Keypair;
    let metadataPDA: anchor.web3.PublicKey;
    let masterEditionPDA: anchor.web3.PublicKey;
    let tokenAccount: anchor.web3.PublicKey;
    let bridgeTokenAccount: any;

    try {
      otdmData = await program.account.contractInfo.fetch(otdm);
      isCollectionExists = true;
    }
    catch (ex) {
      console.log(ex);
    }

    try {
      selfTokensData = await program.account.selfTokenInfo.fetch(selfTokens);
      isUnlock = true;
      bridgeTokenAccount = await getOrCreateTokenAccount(new anchor.web3.PublicKey(selfTokensData.tokenId), bridge);
      console.log(selfTokensData, "asdasdsad " + selfTokensData.tokenId.toString());
      console.log("bridgeTokenAccount", bridgeTokenAccount);

      [otherTokens, otherTokensBump] = await PublicKey.findProgramAddress(
        [new Uint8Array(createHash("SHA256").update(Buffer.from(`other_tokens${selfTokensData.tokenId.toString()}SOL${selfTokensData.contractAddress}`)).digest())],
        program.programId
      );

      [dtom, dtomBump] = await PublicKey.findProgramAddress(
        [new Uint8Array(createHash("SHA256").update(Buffer.from(`duplicate_to_original_mapping${selfTokensData.contractAddress}SOL`)).digest())],
        program.programId
      );
    }
    catch (ex) {
      console.log(ex);
    }
    if (isCollectionExists) {
      // collectionPDA = new anchor.web3.PublicKey(otdmData.contractAddress);
      console.log(otdmData.contractAddress);

      [collectionPDA] = anchor.web3.PublicKey.findProgramAddressSync(
        [new Uint8Array(createHash("SHA256").update(Buffer.from(`${claimData.sourceNftContractAddress}${claimData.sourceChain}`)).digest())],
        program.programId
      );
      console.log("collection ", collectionPDA.toString());


      collectionMetadataPDA = await metaplex
        .nfts()
        .pdas()
        .metadata({ mint: collectionPDA });

      collectionMasterEditionPDA = await metaplex
        .nfts()
        .pdas()
        .masterEdition({ mint: collectionPDA });

      collectionTokenAccount = await spl.getAssociatedTokenAddress(
        collectionPDA,
        wallet.publicKey
      );

      if (selfTokensData) {
        mint = new anchor.web3.PublicKey(selfTokensData.tokenId.toString());

        metadataPDA = await metaplex
          .nfts()
          .pdas()
          .metadata({ mint: mint });

        masterEditionPDA = await metaplex
          .nfts()
          .pdas()
          .masterEdition({ mint: mint });

        tokenAccount = await spl.getAssociatedTokenAddress(
          mint,
          wallet.publicKey
        );
      }
      else {

        newMint = anchor.web3.Keypair.generate();

        [otherTokens, otherTokensBump] = await PublicKey.findProgramAddress(
          [new Uint8Array(createHash("SHA256").update(Buffer.from(`other_tokens${newMint.publicKey.toString()}SOL${collectionPDA.toString()}`)).digest())],
          program.programId
        );

        [dtom, dtomBump] = await PublicKey.findProgramAddress(
          [new Uint8Array(createHash("SHA256").update(Buffer.from(`duplicate_to_original_mapping${collectionPDA.toString()}SOL`)).digest())],
          program.programId
        );

        metadataPDA = await metaplex
          .nfts()
          .pdas()
          .metadata({ mint: newMint.publicKey });

        masterEditionPDA = await metaplex
          .nfts()
          .pdas()
          .masterEdition({ mint: newMint.publicKey });

        tokenAccount = await spl.getAssociatedTokenAddress(
          newMint.publicKey,
          wallet.publicKey
        );
      }
    }
    else {
      console.log("asdsad");
      [collectionPDA] = anchor.web3.PublicKey.findProgramAddressSync(
        [new Uint8Array(createHash("SHA256").update(`${claimData.sourceNftContractAddress}${claimData.sourceChain}`).digest())],
        program.programId
      );

      collectionMetadataPDA = await metaplex
        .nfts()
        .pdas()
        .metadata({ mint: collectionPDA });

      collectionMasterEditionPDA = await metaplex
        .nfts()
        .pdas()
        .masterEdition({ mint: collectionPDA });

      collectionTokenAccount = await spl.getAssociatedTokenAddress(
        collectionPDA,
        wallet.publicKey
      );

      newMint = anchor.web3.Keypair.generate();

      metadataPDA = await metaplex
        .nfts()
        .pdas()
        .metadata({ mint: newMint.publicKey });

      masterEditionPDA = await metaplex
        .nfts()
        .pdas()
        .masterEdition({ mint: newMint.publicKey });

      tokenAccount = await spl.getAssociatedTokenAddress(
        newMint.publicKey,
        wallet.publicKey
      );

      [otherTokens, otherTokensBump] = await PublicKey.findProgramAddress(
        [new Uint8Array(createHash("SHA256").update(Buffer.from(`other_tokens${newMint.publicKey}SOL${collectionPDA.toString()}`)).digest())],
        program.programId
      );

      [dtom, dtomBump] = await PublicKey.findProgramAddress(
        [new Uint8Array(createHash("SHA256").update(Buffer.from(`duplicate_to_original_mapping${collectionPDA.toString()}SOL`)).digest())],
        program.programId
      );
    }

    console.log({
      //   bridge: bridge.toString(),
      //   otherTokens: otherTokens.toString(),
      //   selfTokens: selfTokens.toString(),
      //   originalToDuplicateMapping: otdm.toString(),
      //   duplicateToOriginalMapping: dtom.toString(),
      //   // to: tokenAccount,
      //   user: provider.wallet.publicKey.toString(),
      //   createCollectionMasterEdition: collectionMasterEditionPDA.toString(),
      //   createCollectionMetadataAccount: collectionMetadataPDA.toString(),
      createCollectionMint: collectionPDA.toString(),
      //   createCollectionTokenAccount: collectionTokenAccount.toString(),
      //   // collectionMasterEdition: collectionMasterEditionPDA,
      //   // collectionMetadataAccount: collectionMetadataPDA,
      //   // nftCollectionMint: collectionPDA,
      //   nftMasterEdition: masterEditionPDA.toString(),
      //   nftMetadataAccount: metadataPDA.toString(),
      //   nftMint: mint.toString(),
      //   nftTokenAccount: tokenAccount.toString(),

      //   tokenMetadataProgram: METADATA_PROGRAM_ID.toString(),
      //   systemProgram: SystemProgram.programId.toString(),
      //   // instructionAcc: SYSVAR_INSTRUCTIONS_PUBKEY,
    });
    // console.log("gando");
    console.log(isCollectionExists, isUnlock);
    // console.log(otherTokens.toString());

    try {
      const modifyComputeUnits =
        anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
          units: 600_000,
        });
      let tx: anchor.web3.Transaction;
      if (!isCollectionExists && !isUnlock) {
        console.log("here");
        let data = new ClaimData({
          claimData,
          nftMint: null
        });
        //@ts-ignore
        tx = await program.methods.claimNftWithCollectionCreation(data)
          .accounts({
            bridge: bridge,
            otherTokens: otherTokens,
            selfTokens: selfTokens,
            originalToDuplicateMapping: otdm,
            duplicateToOriginalMapping: dtom,
            createCollectionMasterEdition: collectionMasterEditionPDA,
            createCollectionMetadataAccount: collectionMetadataPDA,
            createCollectionMint: collectionPDA,
            createCollectionTokenAccount: collectionTokenAccount,
            nftMasterEdition: masterEditionPDA,
            nftMetadataAccount: metadataPDA,
            nftMint: newMint.publicKey,
            nftTokenAccount: tokenAccount,
            tokenMetadataProgram: METADATA_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            user: provider.wallet.publicKey,
          })
          .transaction();
      }
      else if (isCollectionExists && !isUnlock) {
        console.log("lllllllllllllllllaaaaaaaaaaaaaaaaaaaaa");
        let data = new ClaimData({
          claimData,
          nftMint: null
        });
        //@ts-ignore
        tx = await program.methods.claimNftWithCreation(data)
          .accounts({
            bridge: bridge,
            otherTokens: otherTokens,
            selfTokens: selfTokens,
            originalToDuplicateMapping: otdm,
            duplicateToOriginalMapping: dtom,
            createCollectionMasterEdition: collectionMasterEditionPDA,
            createCollectionMetadataAccount: collectionMetadataPDA,
            createCollectionMint: collectionPDA,
            nftMasterEdition: masterEditionPDA,
            nftMetadataAccount: metadataPDA,
            nftMint: newMint.publicKey,
            nftTokenAccount: tokenAccount,
            tokenMetadataProgram: METADATA_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            user: provider.wallet.publicKey,
          })
          .transaction();
      }
      else if (isUnlock) {
        let data = new ClaimData({
          claimData,
          nftMint: mint
        });
        //@ts-ignore
        tx = await program.methods.claimNftJustUnlock(data)
          .accounts({
            bridge: bridge,
            otherTokens: otherTokens,
            selfTokens: selfTokens,
            originalToDuplicateMapping: otdm,
            duplicateToOriginalMapping: dtom,
            bridgeTokenAccount: bridgeTokenAccount.address,
            user: provider.wallet.publicKey,
            createCollectionMint: collectionPDA,
            nftTokenAccount: tokenAccount,
            tokenMetadataProgram: METADATA_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID
          })
          .transaction();
      }

      const transferTransaction = new anchor.web3.Transaction().add(
        modifyComputeUnits,
        tx
      );
      let size = getTxSize(transferTransaction, wallet.payer.publicKey);
      console.log("size ::", size);

      const txSig = await anchor.web3.sendAndConfirmTransaction(
        connection,
        transferTransaction,
        [wallet.payer, newMint],
        { skipPreflight: false }
      );

      console.log(txSig);

    }
    catch (ex) {
      //console.log(ex);

      console.dir(ex.logs, { 'maxArrayLength': null });
    }
  });

});

async function getOrCreateTokenAccount(mint: PublicKey, owner: PublicKey) {
  const tokenProgram = Spl.token();
  const program = Spl.associatedToken();

  const [associatedToken] = await PublicKey.findProgramAddress(
    [owner.toBuffer(), tokenProgram.programId.toBuffer(), mint.toBuffer()],
    program.programId
  );

  try {
    const tokenAccount = await tokenProgram.account.token.fetch(
      associatedToken
    );
    return {
      address: associatedToken,
      owner: tokenAccount.authority,
      ...tokenAccount,
    };
  } catch (e) {
    try {
      await program.methods
        .create()
        .accounts({
          mint,
          owner,
          associatedAccount: associatedToken,
        })
        .rpc();

      const tokenAccount = await tokenProgram.account.token.fetch(
        associatedToken
      );
      return {
        address: associatedToken,
        owner: tokenAccount.authority,
        ...tokenAccount,
      };
    } catch (e) {
      throw e;
    }
  }
}