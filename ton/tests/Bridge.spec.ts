import { Address, Dictionary, beginCell, toNano } from '@ton/core';
import { Blockchain, SandboxContract, TreasuryContract } from '@ton/sandbox';
import '@ton/test-utils';
import { AddValidator, Bridge, ClaimData, loadLockedEvent, SignerAndSignature, storeClaimData, storeNewValidator } from '../wrappers/Bridge';
import { KeyPair, mnemonicNew, mnemonicToPrivateKey, sha256_sync, sign } from '@ton/crypto';
import { WalletContractV4 } from '@ton/ton';
import { flattenTransaction } from '@ton/test-utils';
import { testKey } from '@tact-lang/emulator';
import { NftCollection } from '../build/Bridge/tact_NftCollection';
import { TestnetNftCollection } from '../wrappers/ExampleNft';
import { TestnetNftItem } from '../build/ExampleNft/tact_TestnetNftItem';

describe('Bridge', () => {
    let blockchain: Blockchain;
    let deployer: SandboxContract<TreasuryContract>;
    let bridge: SandboxContract<Bridge>;
    let bootstrapValidator: KeyPair;
    let bootstrapValidatorAddress: WalletContractV4;

    beforeEach(async () => {
        blockchain = await Blockchain.create({});
        blockchain.verbosity.debugLogs = true;
        bootstrapValidator = await mnemonicToPrivateKey(await mnemonicNew())
        bootstrapValidatorAddress = WalletContractV4.create({
            publicKey: bootstrapValidator.publicKey,
            workchain: 0
        })


        deployer = await blockchain.treasury('deployer');

        bridge = blockchain.openContract(await Bridge.fromInit(BigInt(`0x${bootstrapValidator.publicKey.toString('hex')}`), bootstrapValidatorAddress.address, "TON"));
        const deployResult = await bridge.send(
            deployer.getSender(),
            {
                value: toNano('1.0'),
            },
            "Deploy"
        );

        expect(deployResult.transactions).toHaveTransaction({
            from: deployer.address,
            to: bridge.address,
            deploy: true,
            success: true,
        });
    });

    it('should deploy', async () => {
    })

    it('should lock nft', async () => {
        const nft = blockchain.openContract(await TestnetNftCollection.fromInit(deployer.address, beginCell().storeInt(1, 32).endCell(), {
            $$type: "RoyaltyParams",
            denominator: 10n,
            destination: deployer.address,
            numerator: 1n,
        }))

        const deploy = await nft.send(deployer.getSender(), {
            value: toNano('0.5'),
            bounce: true
        }, {
            $$type: "Deploy",
            queryId: 453n,
        })

        expect(deploy.transactions).toHaveTransaction({
            from: deployer.address,
            to: nft.address,
            deploy: true, success: true
        })

        const mint = await nft.send(deployer.getSender(), {
            value: toNano('0.1')
        }, {
            $$type: "Mint",
            content: beginCell().endCell(),
            owner: deployer.address,
            token_id: 500n
        })

        const nftItem = blockchain.openContract(TestnetNftItem.fromAddress((await nft.getGetNftAddressByIndex(500n))!))

        const txns = await nftItem.send(deployer.getSender(), {
            value: toNano('2')
        }, {
            $$type: "Transfer",
            custom_payload: null,
            forward_amount: toNano('1'),
            forward_payload: beginCell()
                .storeInt(400, 256)
                .storeAddress(nft.address)
                .storeRef(beginCell().storeStringRefTail("BSC"))
                .storeRef(beginCell().storeStringRefTail("MA MAN"))
                .endCell(),
            new_owner: bridge.address,
            response_destination: bridge.address,
            query_id: 435435n
        })

        let find = txns.transactions.find((e) => {

            for (let i =0; i < e.outMessagesCount; i++) {
                let om = e.outMessages.get(i)!
                if (om.body.asSlice().loadUint(32) === 4205190074) {
                    return true
                }
            }
        })
        let e = find!;
        for (let i = 0; i < e.outMessagesCount; i++) {
            let om = e.outMessages.get(i)!
            if (om.body.asSlice().loadUint(32) === 4205190074) {
                const locked = loadLockedEvent(om.body.asSlice())
                expect(locked.destinationChain.asSlice().loadStringRefTail()).toBe('BSC')
                expect(locked.tokenId).toBe(400n)
                return
            }
        }
        throw new Error('failed')

    })

        it('should add validators', async () => {
            const newValidatorKeypair = testKey("newValidator");
            const wallet = WalletContractV4.create({ workchain: 0, publicKey: newValidatorKeypair.publicKey })
            const nv = {
                $$type: "NewValidator",
                key: BigInt(`0x${newValidatorKeypair.publicKey.toString("hex")}`)
            } as const

            let signature = sign(
                beginCell().store(storeNewValidator(nv)).endCell().hash(),
                bootstrapValidator.secretKey,
            )
            const addValidator: AddValidator = {
                $$type: "AddValidator",
                newValidatorAddress: wallet.address,
                len: 1n,
                newValidatorPublicKey: nv,
                sigs: Dictionary.empty<bigint, SignerAndSignature>().set(0n, {
                    $$type: "SignerAndSignature",
                    key: BigInt(`0x${bootstrapValidator.publicKey.toString('hex')}`),
                    signature: beginCell().storeBuffer(signature).endCell()
                })
            }
            await bridge.send(deployer.getSender(), {
                value: toNano("0.05")
            }, addValidator)
            let validators = await bridge.getValidator(BigInt(`0x${bootstrapValidator.publicKey.toString('hex')}`),)
            let vc = await bridge.getValidatorsCount()
            expect(vc).toBe(2n);
            expect(validators).toBeDefined()
        })


        it('should claim data', async () => {
            let cd: ClaimData = {
                $$type: "ClaimData",
                data1: {
                    $$type: "ClaimData1",
                    destinationChain: "TON",
                    destinationUserAddress: deployer.address,
                    sourceChain: "MATIC",
                    tokenAmount: 1n,
                    tokenId: 0n
                },
                data2: {
                    $$type: "ClaimData2",
                    name: "TestNet",
                    nftType: "singular",
                    symbol: "TNT"
                },
                data3: {
                    $$type: "ClaimData3",
                    fee: toNano("0.1"),
                    metadata: beginCell().storeInt(1, 8).storeStringTail('https://example.org').endCell(),
                    royaltyReceiver: deployer.address,
                    sourceNftContractAddress: beginCell()
                        .storeSlice(
                            beginCell()
                                .storeStringTail('0x0')
                                .endCell()
                                .asSlice(),
                        )
                        .endCell()
                },
                data4: {
                    $$type: 'ClaimData4',
                    newContent: beginCell()
                        .storeInt(0x01, 8)
                        .storeStringRefTail('https://example.org')
                        .endCell(),
                    royalty: {
                        $$type: 'RoyaltyParams',
                        denominator: 10000n,
                        destination: deployer.address,
                        numerator: 10n
                    },
                    transactionHash: '0x0000'
                }
            }
            const signature = sign(
                beginCell().store(storeClaimData(cd)).endCell().hash(),
                bootstrapValidator.secretKey,
            );

            const txns = await bridge.send(deployer.getSender(), {
                value: toNano("0.2")
            }, {
                $$type: "ClaimNFT721",
                data: cd,
                len: 1n,
                signatures: Dictionary.empty<bigint, SignerAndSignature>().set(0n, {
                    $$type: "SignerAndSignature",
                    key: BigInt(`0x${bootstrapValidator.publicKey.toString('hex')}`),
                    signature: beginCell().storeBuffer(signature).endCell()
                })
            })

            {
                let cd: ClaimData = {
                    $$type: "ClaimData",
                    data1: {
                        $$type: "ClaimData1",
                        destinationChain: "TON",
                        destinationUserAddress: deployer.address,
                        sourceChain: "MATIC",
                        tokenAmount: 1n,
                        tokenId: 1n
                    },
                    data2: {
                        $$type: "ClaimData2",
                        name: "TestNet",
                        nftType: "singular",
                        symbol: "TNT"
                    },
                    data3: {
                        $$type: "ClaimData3",
                        fee: toNano("0.1"),
                        metadata: beginCell().storeInt(1, 8).storeStringTail('https://example.org').endCell(),
                        royaltyReceiver: deployer.address,
                        sourceNftContractAddress: beginCell()
                            .storeSlice(
                                beginCell()
                                    .storeStringTail('0x15')
                                    .endCell()
                                    .asSlice(),
                            )
                            .endCell()
                    },
                    data4: {
                        $$type: 'ClaimData4',
                        newContent: beginCell()
                            .storeInt(0x01, 8)
                            .storeStringRefTail('https://example.org')
                            .endCell(),
                        royalty: {
                            $$type: 'RoyaltyParams',
                            denominator: 10000n,
                            destination: deployer.address,
                            numerator: 10n
                        },
                        transactionHash: '0x0000'
                    }
                }
                const signature = sign(
                    beginCell().store(storeClaimData(cd)).endCell().hash(),
                    bootstrapValidator.secretKey,
                );

                const txns = await bridge.send(deployer.getSender(), {
                    value: toNano("0.2")
                }, {
                    $$type: "ClaimNFT721",
                    data: cd,
                    len: 1n,
                    signatures: Dictionary.empty<bigint, SignerAndSignature>().set(0n, {
                        $$type: "SignerAndSignature",
                        key: BigInt(`0x${bootstrapValidator.publicKey.toString('hex')}`),
                        signature: beginCell().storeBuffer(signature).endCell()
                    })
                })
            }
        })

    });

    const toKey = (key: string) => {
        return BigInt(`0x${sha256_sync(key).toString('hex')}`);
    };