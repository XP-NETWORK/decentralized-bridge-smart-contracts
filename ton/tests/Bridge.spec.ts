import { Dictionary, beginCell, toNano } from '@ton/core';
import { Blockchain, SandboxContract, TreasuryContract } from '@ton/sandbox';
import '@ton/test-utils';
import { Bridge, ClaimData, SignerAndSignature, storeClaimData } from '../wrappers/Bridge';
import { KeyPair, mnemonicNew, mnemonicToPrivateKey, sha256_sync, sign } from '@ton/crypto';
import { WalletContractV4 } from '@ton/ton';
import { flattenTransaction } from '@ton/test-utils';

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
        console.log(`Deployed Successfully`)
    })

    it('should receive installment', async () => {
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
        console.log(txns.transactions.map(flattenTransaction))
    })

});

const toKey = (key: string) => {
    return BigInt(`0x${sha256_sync(key).toString('hex')}`);
};