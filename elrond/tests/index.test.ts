import { test, beforeEach, afterEach, it, describe, expect } from "vitest";
import { assertAccount, SWorld, SWallet, SContract, e, Wallet, } from "xsuite";
import { AddressEncodable } from "xsuite/dist/data/AddressEncodable";
import { UserSigner } from "xsuite/dist/world/signer";

let world: SWorld;
let deployer: SWallet;
let contract: SContract;
let validator1: SWallet;
let validator2: SWallet;
let receiver: SWallet;

let nft_collection = "NFT-123456";
let sft_collection = "SFT-123456";

let validator1Address = e.Addr("erd1m229kx85t9jsamjuxpu6sjtu6jws7q4lesne9m5gdex9g8ps6n9scwk2v0");
let validator2Address = e.Addr("erd1szew9rj8w2s9ulry7tld89wt002zea0ywkwwfw5xsz07hwsrdgrqzlz5ql");
let correctSignature = "86a44e6929c3f599e00c5cf9bc9dc876697df19d55ec76cd4f32482b1ee72a2a7363c04912e73a96a2d10b24ccff180dbe861aea955df00e3ecd0d5c326d5a08";
let badSignature = "86a34e6929c3f599e00c5cf9bc9dc876697df19d55ec76cd4f32482b1ee72a2a7363c04912e73a96a2d10b24ccff180dbe861aea955df00e3ecd0d5c326d5a08";

beforeEach(async () => {
    world = await SWorld.start();
    deployer = await world.createWallet({ balance: 15 });
    validator1 = await world.createWallet({
        balance: 15,
        kvs: [
            e.kvs.Esdts(
                [
                    { id: nft_collection, nonce: 1, amount: 1, name: "Nft Name", },
                    { id: sft_collection, nonce: 1, amount: 3, name: "Sft Name", },
                ]
            )
        ],
    });
    validator2 = await world.createWallet({
        balance: 15,
        kvs: [
            e.kvs.Esdts(
                [
                    { id: nft_collection, nonce: 2, amount: 1, name: "Nft Name", },
                    { id: sft_collection, nonce: 2, amount: 3, name: "Sft Name", },
                ]
            )
        ],
    });
    receiver = await world.createWallet({ balance: 15 });

    ({ contract } = await deployer.deployContract({
        code: "file:output/bridge.wasm",
        codeMetadata: [],
        gasLimit: 20_000_000,
        codeArgs: [validator1Address],
    }));
});

afterEach(async () => {
    await world.terminate();
});
describe("Bridge", async () => {
    describe("Deployment", async () => {
        it("Should set validator correctly", async () => {
            assertAccount(await contract.getAccountWithKvs(), {
                hasKvs: [e.kvs.Mapper("validators", validator1Address).Vec([
                    e.Tuple(
                        e.Bool(true),
                        e.U32(BigInt(0)),
                    ),
                ])],
            });
        });
        it("Should have the correct validators count", async () => {
            assertAccount(await contract.getAccountWithKvs(), {
                hasKvs: [e.kvs.Mapper("validatorsCount").Value(e.U64(1))],
            });
        });
    });
    describe("claimValidatorRewards", async () => {
        it("Should fail if no signatures are provided", async () => {
            await validator1.callContract({
                callee: contract,
                gasLimit: 50_000_000,
                funcName: "claimValidatorRewards",
                funcArgs: [
                    validator2Address,
                    e.List()
                ],
            }).assertFail({ code: 4, message: "Must have signatures!" });
        });
        it("Should fail if validators do not reach threshold", async () => {
            await validator1.callContract({
                callee: contract,
                gasLimit: 50_000_000,
                funcName: "claimValidatorRewards",
                funcArgs: [
                    validator1Address,
                    e.List(
                        e.Tuple(validator1Address,
                            e.Buffer(correctSignature))
                    )
                ],
            }).assertFail({ code: 4, message: "Threshold not reached!" });
        });
        // it("should successfully transfer funds when validator claims rewards", async () => {
        // })
    });
    describe("addValidator", async () => {
        it("Should add new validator and check the validators count", async () => {
            const result = await validator1.callContract({
                callee: contract,
                gasLimit: 50_000_000,
                funcName: "addValidator",
                funcArgs: [
                    validator2Address,
                    e.List(
                        e.Tuple(validator1Address,
                            e.Buffer(correctSignature))
                    )
                ],
            });
            assertAccount(await contract.getAccountWithKvs(), {
                hasKvs: [e.kvs.Mapper("validatorsCount").Value(e.U64(2))],
            });
        });
        it("Should fail if no signatures are provided", async () => {
            await validator1.callContract({
                callee: contract,
                gasLimit: 50_000_000,
                funcName: "addValidator",
                funcArgs: [
                    validator2Address,
                    e.List()
                ],
            }).assertFail({ code: 4, message: "Must have signatures!" });
        });
        it("Should fail if validators do not reach threshold", async () => {
            await validator1.callContract({
                callee: contract,
                gasLimit: 50_000_000,
                funcName: "addValidator",
                funcArgs: [
                    validator2Address,
                    e.List(
                        e.Tuple(validator1Address,
                            e.Buffer(badSignature))
                    )
                ],
            }).assertFail({ code: 4, message: "Threshold not reached!" });
        });
    });
    describe("lock721", async () => {
        it("should fail to lock 721 NFT if amount is zero", async () => {
            let data = [
                e.Buffer(Buffer.from(nft_collection)),
                e.Buffer(Buffer.from("BSC")),
                e.Buffer(Buffer.from("0x6f7C0c6A6dd6E435b0EEc1c9F7Bce01A1908f386")),
                e.Buffer(Buffer.from(nft_collection)),
                e.U64(1)
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                funcName: "lock721",
                funcArgs: data,
            }).assertFail({ code: 4, message: "No nft received" });
        })
        it("should fail to lock 721 NFT if no nft received", async () => {
            let data = [
                e.Buffer(Buffer.from(nft_collection)),
                e.Buffer(Buffer.from("BSC")),
                e.Buffer(Buffer.from("0x6f7C0c6A6dd6E435b0EEc1c9F7Bce01A1908f386")),
                e.Buffer(Buffer.from(nft_collection)),
                e.U64(1)
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                funcName: "lock721",
                funcArgs: data,
            }).assertFail({ code: 4, message: "No nft received" });
        });
        it("should lock 721 NFT", async () => {
            let data = [
                e.Buffer(Buffer.from(nft_collection)),
                e.Buffer(Buffer.from("BSC")),
                e.Buffer(Buffer.from("0x6f7C0c6A6dd6E435b0EEc1c9F7Bce01A1908f386")),
                e.Buffer(Buffer.from(nft_collection)),
                e.U64(1)
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                esdts: [{ id: nft_collection, nonce: 1, amount: 1 }],
                funcName: "lock721",
                funcArgs: data,
            });
        });
    });
    describe("lock1155", async () => {
        it("should fail to lock 1155 NFT if amount is zero", async () => {
            let data = [
                e.Buffer(Buffer.from(sft_collection)),
                e.Buffer(Buffer.from("BSC")),
                e.Buffer(Buffer.from("0x6f7C0c6A6dd6E435b0EEc1c9F7Bce01A1908f386")),
                e.Buffer(Buffer.from(sft_collection)),
                e.I(BigInt(3)),
                e.U64(1)
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                funcName: "lock1155",
                funcArgs: data,
            }).assertFail({ code: 4, message: "No sft received" });
        })
        it("should fail to lock 1155 NFT if no nft received", async () => {
            let data = [
                e.Buffer(Buffer.from(sft_collection)),
                e.Buffer(Buffer.from("BSC")),
                e.Buffer(Buffer.from("0x6f7C0c6A6dd6E435b0EEc1c9F7Bce01A1908f386")),
                e.Buffer(Buffer.from(sft_collection)),
                e.I(BigInt(3)),
                e.U64(1)
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                funcName: "lock1155",
                funcArgs: data,
            }).assertFail({ code: 4, message: "No sft received" });
        });
        it("should lock 1155 NFT", async () => {
            let data = [
                e.Buffer(Buffer.from(sft_collection)),
                e.Buffer(Buffer.from("BSC")),
                e.Buffer(Buffer.from("0x6f7C0c6A6dd6E435b0EEc1c9F7Bce01A1908f386")),
                e.Buffer(Buffer.from(sft_collection)),
                e.I(BigInt(3)),
                e.U64(1)
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                esdts: [{ id: sft_collection, nonce: 1, amount: 3 }],
                funcName: "lock1155",
                funcArgs: data,
            });
        });
    });
    describe("claimNFT721", async () => {
        it("should fail to claim 721 if msg.value less than data.fee", async () => {
            let data = [
                e.Tuple(
                    e.Buffer(new Uint8Array(Buffer.from(Number("36").toString(16), "hex"))),
                    e.Buffer(new Uint8Array(Buffer.from("BSC"))),
                    e.Buffer(new Uint8Array(Buffer.from("MULTIVERSX"))),
                    e.Addr(receiver.toString()),
                    e.Buffer(new Uint8Array(Buffer.from("0x17e44f7014979816a3819daa966cc28f70344198"))),
                    e.Buffer(new Uint8Array(Buffer.from("v3Test"))),
                    e.Buffer(new Uint8Array(Buffer.from("V3T"))),
                    e.U(BigInt("1000")),
                    e.Addr(receiver.toString()),
                    e.Buffer(new Uint8Array(Buffer.from("v3t/1"))),
                    e.Buffer(new Uint8Array(Buffer.from("0x0c9e74f00a7ec0533a609c9dcf7ab55dc50516aaf09712ace4d91d3b8af4d6f7"))),
                    e.U(BigInt("1")),
                    e.Buffer(new Uint8Array(Buffer.from("singular"))),
                    e.U(BigInt("100000000000000")),
                ),
                e.List(e.Tuple(
                    e.Addr("erd1yycd9st0jx0kxn0gg7qpehxwlw7plzda6ffy6kmtjnkmlqsm9vqqmd070h"),
                    e.Buffer("4774706ed157439948ccd82e21ceb702fd02c2fe96e6cfe812caff57f842b82adb6be2b1633cfae647d5a85bcaa640db3d2a6abe90703c636e16fb45be6b9702")
                )),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmUBFTnxZpaM7xrJ62Z9kNi3dfQwEWPQhthsnXdLEjJhDb/9999.png", 'utf-8'))),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmSaY9zZnKWGa8jmMFNN6LrDGykjSiryUz8YeUjjJ97A8w/9999.json", 'utf-8')))
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                value: BigInt("0"),
                funcName: "claimNft721",
                funcArgs: data,
            }).assertFail({ code: 4, message: "Fee and sent amount do not match" });
        });
        it("Should fail to claim 721 if current chain is not equal to data.chain", async () => {
            let data = [
                e.Tuple(
                    e.Buffer(new Uint8Array(Buffer.from(Number("36").toString(16), "hex"))),
                    e.Buffer(new Uint8Array(Buffer.from("BSC"))),
                    e.Buffer(new Uint8Array(Buffer.from("MULTIVERS"))),
                    e.Addr(receiver.toString()),
                    e.Buffer(new Uint8Array(Buffer.from("0x17e44f7014979816a3819daa966cc28f70344198"))),
                    e.Buffer(new Uint8Array(Buffer.from("v3Test"))),
                    e.Buffer(new Uint8Array(Buffer.from("V3T"))),
                    e.U(BigInt("1000")),
                    e.Addr(receiver.toString()),
                    e.Buffer(new Uint8Array(Buffer.from("v3t/1"))),
                    e.Buffer(new Uint8Array(Buffer.from("0x0c9e74f00a7ec0533a609c9dcf7ab55dc50516aaf09712ace4d91d3b8af4d6f7"))),
                    e.U(BigInt("1")),
                    e.Buffer(new Uint8Array(Buffer.from("singular"))),
                    e.U(BigInt("1")),
                ),
                e.List(e.Tuple(
                    e.Addr("erd1yycd9st0jx0kxn0gg7qpehxwlw7plzda6ffy6kmtjnkmlqsm9vqqmd070h"),
                    e.Buffer("4774706ed157439948ccd82e21ceb702fd02c2fe96e6cfe812caff57f842b82adb6be2b1633cfae647d5a85bcaa640db3d2a6abe90703c636e16fb45be6b9702")
                )),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmUBFTnxZpaM7xrJ62Z9kNi3dfQwEWPQhthsnXdLEjJhDb/9999.png", 'utf-8'))),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmSaY9zZnKWGa8jmMFNN6LrDGykjSiryUz8YeUjjJ97A8w/9999.json", 'utf-8')))
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                value: BigInt("1"),
                funcName: "claimNft721",
                funcArgs: data,
            }).assertFail({ code: 4, message: "Invalid destination chain" });
        });
        it("Should fail to claim 721 if data already processed", async () => {
            let data = [
                e.Tuple(
                    e.Buffer(new Uint8Array(Buffer.from(Number("36").toString(16), "hex"))),
                    e.Buffer(new Uint8Array(Buffer.from("BSC"))),
                    e.Buffer(new Uint8Array(Buffer.from("MULTIVERSX"))),
                    validator1Address,
                    e.Buffer(new Uint8Array(Buffer.from("0x17e44f7014979816a3819daa966cc28f70344198"))),
                    e.Buffer(new Uint8Array(Buffer.from("v3Test"))),
                    e.Buffer(new Uint8Array(Buffer.from("V3T"))),
                    e.U(BigInt("1000")),
                    e.Addr("erd1yycd9st0jx0kxn0gg7qpehxwlw7plzda6ffy6kmtjnkmlqsm9vqqmd070h"),
                    e.Buffer(new Uint8Array(Buffer.from("v3t/1"))),
                    e.Buffer(new Uint8Array(Buffer.from("0x0c9e74f00a7ec0533a609c9dcf7ab55dc50516aaf09712ace4d91d3b8af4d6f7"))),
                    e.U(BigInt("1")),
                    e.Buffer(new Uint8Array(Buffer.from("singular"))),
                    e.U(BigInt("1")), // fee
                ),
                e.List(e.Tuple(
                    validator1Address,
                    e.Buffer("8fd432ceb34882fc2667a7e6f0d7ba84bab2053a96f55b80431ae729feb516f03138842edb0e99ba3f42d524260073d3cd4df003ff96aae785df30a134a5e202")
                )),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmUBFTnxZpaM7xrJ62Z9kNi3dfQwEWPQhthsnXdLEjJhDb/9999.png", 'utf-8'))),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmSaY9zZnKWGa8jmMFNN6LrDGykjSiryUz8YeUjjJ97A8w/9999.json", 'utf-8')))
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                value: 1,
                funcName: "claimNft721",
                funcArgs: data,
            });
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                value: 1,
                funcName: "claimNft721",
                funcArgs: data,
            }).assertFail({ code: 4, message: "Data already processed!" });
        });
        it("Should fail to claim 721 if data.nftType is not according to called function", async () => {
            let data = [
                e.Tuple(
                    e.Buffer(new Uint8Array(Buffer.from(Number("36").toString(16), "hex"))),
                    e.Buffer(new Uint8Array(Buffer.from("BSC"))),
                    e.Buffer(new Uint8Array(Buffer.from("MULTIVERSX"))),
                    validator1Address,
                    e.Buffer(new Uint8Array(Buffer.from("0x17e44f7014979816a3819daa966cc28f70344198"))),
                    e.Buffer(new Uint8Array(Buffer.from("v3Test"))),
                    e.Buffer(new Uint8Array(Buffer.from("V3T"))),
                    e.U(BigInt("1000")),
                    e.Addr("erd1yycd9st0jx0kxn0gg7qpehxwlw7plzda6ffy6kmtjnkmlqsm9vqqmd070h"),
                    e.Buffer(new Uint8Array(Buffer.from("v3t/1"))),
                    e.Buffer(new Uint8Array(Buffer.from("0x0c9e74f00a7ec0533a609c9dcf7ab55dc50516aaf09712ace4d91d3b8af4d6f7"))),
                    e.U(BigInt("1")),
                    e.Buffer(new Uint8Array(Buffer.from("multiple"))),
                    e.U(BigInt("1")), // fee
                ),
                e.List(e.Tuple(
                    validator1Address,
                    e.Buffer("8fd432ceb34882fc2667a7e6f0d7ba84bab2053a96f55b80431ae729feb516f03138842edb0e99ba3f42d524260073d3cd4df003ff96aae785df30a134a5e202")
                )),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmUBFTnxZpaM7xrJ62Z9kNi3dfQwEWPQhthsnXdLEjJhDb/9999.png", 'utf-8'))),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmSaY9zZnKWGa8jmMFNN6LrDGykjSiryUz8YeUjjJ97A8w/9999.json", 'utf-8')))
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                value: 1,
                funcName: "claimNft721",
                funcArgs: data,
            }).assertFail({ code: 4, message: "Invalid NFT type!" });
        });
        it("Should fail to claim 721 if threshold not reached", async () => {
            let data = [
                e.Tuple(
                    e.Buffer(new Uint8Array(Buffer.from(Number("36").toString(16), "hex"))),
                    e.Buffer(new Uint8Array(Buffer.from("BSC"))),
                    e.Buffer(new Uint8Array(Buffer.from("MULTIVERSX"))),
                    validator1Address,
                    e.Buffer(new Uint8Array(Buffer.from("0x17e44f7014979816a3819daa966cc28f70344198"))),
                    e.Buffer(new Uint8Array(Buffer.from("v3Test"))),
                    e.Buffer(new Uint8Array(Buffer.from("V3T"))),
                    e.U(BigInt("1000")),
                    e.Addr("erd1yycd9st0jx0kxn0gg7qpehxwlw7plzda6ffy6kmtjnkmlqsm9vqqmd070h"),
                    e.Buffer(new Uint8Array(Buffer.from("v3t/1"))),
                    e.Buffer(new Uint8Array(Buffer.from("0x0c9e74f00a7ec0533a609c9dcf7ab55dc50516aaf09712ace4d91d3b8af4d6f7"))),
                    e.U(BigInt("1")),
                    e.Buffer(new Uint8Array(Buffer.from("singular"))),
                    e.U(BigInt("1")), // fee
                ),
                e.List(e.Tuple(
                    validator1Address,
                    e.Buffer("8fe432ceb34882fc2667a7e6f0d7ba84bab2053a96f55b80431ae729feb516f03138842edb0e99ba3f42d524260073d3cd4df003ff96aae785df30a134a5e202")
                )),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmUBFTnxZpaM7xrJ62Z9kNi3dfQwEWPQhthsnXdLEjJhDb/9999.png", 'utf-8'))),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmSaY9zZnKWGa8jmMFNN6LrDGykjSiryUz8YeUjjJ97A8w/9999.json", 'utf-8')))
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                value: 1,
                funcName: "claimNft721",
                funcArgs: data,
            }).assertFail({ code: 4, message: "Threshold not reached!" });
        });
    });
    describe("claimNFT1155", async () => {
        it("should fail to claim 1155 if msg.value less than data.fee", async () => {
            let data = [
                e.Tuple(
                    e.Buffer(new Uint8Array(Buffer.from(Number("36").toString(16), "hex"))),
                    e.Buffer(new Uint8Array(Buffer.from("BSC"))),
                    e.Buffer(new Uint8Array(Buffer.from("MULTIVERSX"))),
                    e.Addr(receiver.toString()),
                    e.Buffer(new Uint8Array(Buffer.from("0x17e44f7014979816a3819daa966cc28f70344198"))),
                    e.Buffer(new Uint8Array(Buffer.from("v3Test"))),
                    e.Buffer(new Uint8Array(Buffer.from("V3T"))),
                    e.U(BigInt("1000")),
                    e.Addr(receiver.toString()),
                    e.Buffer(new Uint8Array(Buffer.from("v3t/1"))),
                    e.Buffer(new Uint8Array(Buffer.from("0x0c9e74f00a7ec0533a609c9dcf7ab55dc50516aaf09712ace4d91d3b8af4d6f7"))),
                    e.U(BigInt("1")),
                    e.Buffer(new Uint8Array(Buffer.from("multiple"))),
                    e.U(BigInt("100000000000000")),
                ),
                e.List(e.Tuple(
                    e.Addr("erd1yycd9st0jx0kxn0gg7qpehxwlw7plzda6ffy6kmtjnkmlqsm9vqqmd070h"),
                    e.Buffer("4774706ed157439948ccd82e21ceb702fd02c2fe96e6cfe812caff57f842b82adb6be2b1633cfae647d5a85bcaa640db3d2a6abe90703c636e16fb45be6b9702")
                )),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmUBFTnxZpaM7xrJ62Z9kNi3dfQwEWPQhthsnXdLEjJhDb/9999.png", 'utf-8'))),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmSaY9zZnKWGa8jmMFNN6LrDGykjSiryUz8YeUjjJ97A8w/9999.json", 'utf-8')))
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                value: BigInt("0"),
                funcName: "claimNft1155",
                funcArgs: data,
            }).assertFail({ code: 4, message: "Fee and sent amount do not match" });
        });
        it("Should fail to claim 1155 if current chain is not equal to data.chain", async () => {
            let data = [
                e.Tuple(
                    e.Buffer(new Uint8Array(Buffer.from(Number("36").toString(16), "hex"))),
                    e.Buffer(new Uint8Array(Buffer.from("BSC"))),
                    e.Buffer(new Uint8Array(Buffer.from("MULTIVERS"))),
                    e.Addr(receiver.toString()),
                    e.Buffer(new Uint8Array(Buffer.from("0x17e44f7014979816a3819daa966cc28f70344198"))),
                    e.Buffer(new Uint8Array(Buffer.from("v3Test"))),
                    e.Buffer(new Uint8Array(Buffer.from("V3T"))),
                    e.U(BigInt("1000")),
                    e.Addr(receiver.toString()),
                    e.Buffer(new Uint8Array(Buffer.from("v3t/1"))),
                    e.Buffer(new Uint8Array(Buffer.from("0x0c9e74f00a7ec0533a609c9dcf7ab55dc50516aaf09712ace4d91d3b8af4d6f7"))),
                    e.U(BigInt("1")),
                    e.Buffer(new Uint8Array(Buffer.from("singular"))),
                    e.U(BigInt("1")),
                ),
                e.List(e.Tuple(
                    e.Addr("erd1yycd9st0jx0kxn0gg7qpehxwlw7plzda6ffy6kmtjnkmlqsm9vqqmd070h"),
                    e.Buffer("4774706ed157439948ccd82e21ceb702fd02c2fe96e6cfe812caff57f842b82adb6be2b1633cfae647d5a85bcaa640db3d2a6abe90703c636e16fb45be6b9702")
                )),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmUBFTnxZpaM7xrJ62Z9kNi3dfQwEWPQhthsnXdLEjJhDb/9999.png", 'utf-8'))),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmSaY9zZnKWGa8jmMFNN6LrDGykjSiryUz8YeUjjJ97A8w/9999.json", 'utf-8')))
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                value: BigInt("1"),
                funcName: "claimNft1155",
                funcArgs: data,
            }).assertFail({ code: 4, message: "Invalid destination chain" });
        });
        it("Should fail to claim 1155 if data already processed", async () => {
            let data = [
                e.Tuple(
                    e.Buffer(new Uint8Array(Buffer.from(Number("36").toString(16), "hex"))),
                    e.Buffer(new Uint8Array(Buffer.from("BSC"))),
                    e.Buffer(new Uint8Array(Buffer.from("MULTIVERSX"))),
                    validator1Address,
                    e.Buffer(new Uint8Array(Buffer.from("0x17e44f7014979816a3819daa966cc28f70344198"))),
                    e.Buffer(new Uint8Array(Buffer.from("v4Test"))),
                    e.Buffer(new Uint8Array(Buffer.from("V4T"))),
                    e.U(BigInt("1000")),
                    e.Addr("erd1yycd9st0jx0kxn0gg7qpehxwlw7plzda6ffy6kmtjnkmlqsm9vqqmd070h"),
                    e.Buffer(new Uint8Array(Buffer.from("v4t/1"))),
                    e.Buffer(new Uint8Array(Buffer.from("0x0c9e74f00a7ec0533a609c9dcf7ab55dc50516aaf09712ace4d91d3b8af4d6f7"))),
                    e.U(BigInt("1")),
                    e.Buffer(new Uint8Array(Buffer.from("multiple"))),
                    e.U(BigInt("1")), // fee
                ),
                e.List(e.Tuple(
                    validator1Address,
                    e.Buffer("004a629edaa3e425ea73bc52cd7ad566b768bc4d195f9c31bf4f035f44c43a946f8293e7562520f49821d63a3d2220378846f3fac85c4e5a628ed22e43f6e509")
                )),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmUBFTnxZpaM7xrJ62Z9kNi3dfQwEWPQhthsnXdLEjJhDb/9999.png", 'utf-8'))),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmSaY9zZnKWGa8jmMFNN6LrDGykjSiryUz8YeUjjJ97A8w/9999.json", 'utf-8')))
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                value: 1,
                funcName: "claimNft1155",
                funcArgs: data,
            });
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                value: 1,
                funcName: "claimNft1155",
                funcArgs: data,
            }).assertFail({ code: 4, message: "Data already processed!" });
        });
        it("Should fail to claim 1155 if data.nftType is not according to called function", async () => {
            let data = [
                e.Tuple(
                    e.Buffer(new Uint8Array(Buffer.from(Number("36").toString(16), "hex"))),
                    e.Buffer(new Uint8Array(Buffer.from("BSC"))),
                    e.Buffer(new Uint8Array(Buffer.from("MULTIVERSX"))),
                    validator1Address,
                    e.Buffer(new Uint8Array(Buffer.from("0x17e44f7014979816a3819daa966cc28f70344198"))),
                    e.Buffer(new Uint8Array(Buffer.from("v3Test"))),
                    e.Buffer(new Uint8Array(Buffer.from("V3T"))),
                    e.U(BigInt("1000")),
                    e.Addr("erd1yycd9st0jx0kxn0gg7qpehxwlw7plzda6ffy6kmtjnkmlqsm9vqqmd070h"),
                    e.Buffer(new Uint8Array(Buffer.from("v3t/1"))),
                    e.Buffer(new Uint8Array(Buffer.from("0x0c9e74f00a7ec0533a609c9dcf7ab55dc50516aaf09712ace4d91d3b8af4d6f7"))),
                    e.U(BigInt("1")),
                    e.Buffer(new Uint8Array(Buffer.from("singular"))),
                    e.U(BigInt("1")), // fee
                ),
                e.List(e.Tuple(
                    validator1Address,
                    e.Buffer("8fd432ceb34882fc2667a7e6f0d7ba84bab2053a96f55b80431ae729feb516f03138842edb0e99ba3f42d524260073d3cd4df003ff96aae785df30a134a5e202")
                )),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmUBFTnxZpaM7xrJ62Z9kNi3dfQwEWPQhthsnXdLEjJhDb/9999.png", 'utf-8'))),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmSaY9zZnKWGa8jmMFNN6LrDGykjSiryUz8YeUjjJ97A8w/9999.json", 'utf-8')))
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                value: 1,
                funcName: "claimNft1155",
                funcArgs: data,
            }).assertFail({ code: 4, message: "Invalid NFT type!" });
        });
        it("Should fail to claim 1155 if threshold not reached", async () => {
            let data = [
                e.Tuple(
                    e.Buffer(new Uint8Array(Buffer.from(Number("36").toString(16), "hex"))),
                    e.Buffer(new Uint8Array(Buffer.from("BSC"))),
                    e.Buffer(new Uint8Array(Buffer.from("MULTIVERSX"))),
                    validator1Address,
                    e.Buffer(new Uint8Array(Buffer.from("0x17e44f7014979816a3819daa966cc28f70344198"))),
                    e.Buffer(new Uint8Array(Buffer.from("v3Test"))),
                    e.Buffer(new Uint8Array(Buffer.from("V3T"))),
                    e.U(BigInt("1000")),
                    e.Addr("erd1yycd9st0jx0kxn0gg7qpehxwlw7plzda6ffy6kmtjnkmlqsm9vqqmd070h"),
                    e.Buffer(new Uint8Array(Buffer.from("v3t/1"))),
                    e.Buffer(new Uint8Array(Buffer.from("0x0c9e74f00a7ec0533a609c9dcf7ab55dc50516aaf09712ace4d91d3b8af4d6f7"))),
                    e.U(BigInt("1")),
                    e.Buffer(new Uint8Array(Buffer.from("multiple"))),
                    e.U(BigInt("1")), // fee
                ),
                e.List(e.Tuple(
                    validator1Address,
                    e.Buffer("25d4b41c59f712ed7b5d5ba3930babc3c39504270e983a008387217b381114ef9b45bee61344a762c5fe63e8261bc5ed18c9d25fba445ea5364a0cfbfc22680d")
                )),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmUBFTnxZpaM7xrJ62Z9kNi3dfQwEWPQhthsnXdLEjJhDb/9999.png", 'utf-8'))),
                e.Buffer(new Uint8Array(Buffer.from("https://ipfs.io/ipfs/QmSaY9zZnKWGa8jmMFNN6LrDGykjSiryUz8YeUjjJ97A8w/9999.json", 'utf-8')))
            ];
            await validator1.callContract({
                callee: contract,
                gasLimit: 600_000_000,
                value: 1,
                funcName: "claimNft1155",
                funcArgs: data,
            }).assertFail({ code: 4, message: "Threshold not reached!" });
        });
    });
});