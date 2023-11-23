import { Bridge, ClaimData, SignerAndSignature, storeClaimData } from "../wrappers/Bridge";
import { ContractSystem, testKey } from "@tact-lang/emulator";
import { Address, beginCell, toNano } from "ton-core";
import { sign } from "ton-crypto";
import { NFTStorageDeployer } from "../build/Bridge/tact_NFTStorageDeployer";
import { NFTCollectionDeployer } from "../build/Bridge/tact_NFTCollectionDeployer";
import { Gando, Gnado, storeGnado } from "../build/Bridge/tact_Gando";
import { Dictionary } from "ton";
import { NftCollection } from "../build/Bridge/tact_NftCollection";
import { NFTStorageERC721 } from "../build/Bridge/tact_NFTStorageERC721";

describe('wallet', () => {
    it('should deploy', async () => {

        // Create wallet
        let key = testKey('muadsfj');
        let publicKey = beginCell().storeBuffer(key.publicKey).endCell().beginParse().loadUintBig(256);
        let system = await ContractSystem.create();
        let treasure = system.treasure('treasure');
        let g: Gnado = {
            $$type: "Gnado", address: Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address, chain: "TON"
        }
        let contract = system.open(await Bridge.fromInit(publicKey, Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address, "TON"));
        let storageDeployer = system.open(await NFTStorageDeployer.fromInit(contract.address));
        let storage = system.open(await NFTStorageERC721.fromInit(Address.parseFriendly("kQDsNFk2TW9PNyfLcjY34fFsmz2-z2oWmPQdzp5Bhm_IPjBC").address, contract.address));
        let tracker = system.track(contract.address);
        let logger = system.log(contract.address);
        let storageDeployerTracker = system.track(storageDeployer.address);
        let storageTracker = system.track(storage.address);

        await contract.send(treasure, { value: toNano('10') }, 'Deploy');
        // await system.run();

        // await contract.send(
        //     treasure,
        //     {
        //         value: toNano('1.5')
        //     },
        //     {
        //         $$type: "Lock721",
        //         destinationChain: "BSC",
        //         destinationUserAddress: "0x6f7C0c6A6dd6E435b0EEc1c9F7Bce01A1908f386",
        //         sourceNftContractAddress: Address.parseFriendly("kQDsNFk2TW9PNyfLcjY34fFsmz2-z2oWmPQdzp5Bhm_IPjBC").address,
        //         tokenId: 0n
        //     });
        // await system.run();

        // console.log(await contract.getOriginal721Mapping(Address.parseFriendly("kQDsNFk2TW9PNyfLcjY34fFsmz2-z2oWmPQdzp5Bhm_IPjBC").address, "TON"));
        // console.log(await contract.getDuplicate721Mapping(Address.parseFriendly("kQDsNFk2TW9PNyfLcjY34fFsmz2-z2oWmPQdzp5Bhm_IPjBC").address));


        // console.log(await contract.getOriginalToDuplicate("kQDsNFk2TW9PNyfLcjY34fFsmz2-z2oWmPQdzp5Bhm_IPjBC", "TON"));
        // console.log(await contract.getDuplicateToOriginal(Address.parseFriendly("kQDsNFk2TW9PNyfLcjY34fFsmz2-z2oWmPQdzp5Bhm_IPjBC").address));



        const OFFCHAIN_CONTENT_PREFIX = 0x01;

        const string_first = "https://s.getgems.io/nft-staging/c/628f6ab8077060a7a8d52d63/"; // Change to the content URL you prepared

        let newContent = beginCell().storeInt(0x01, 8).storeStringRefTail("https://s.getgems.io/nft-staging/c/628f6ab8077060a7a8d52d63/").endCell();

        // The Transaction body we want to pass to the smart contract
        let body = beginCell().storeUint(0, 32).storeStringTail("Mint").endCell();




        let claimData: ClaimData = {
            $$type: "ClaimData",
            data1: {
                $$type: 'ClaimData1',
                tokenId: 64n,
                sourceChain: 'BSC',
                destinationChain: 'TON',
                destinationUserAddress: Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address,
                tokenAmount: 1n
            },
            data2: {
                $$type: 'ClaimData2',
                name: 'Gando',
                symbol: 'Gando',
                nftType: 'singular',
            },
            data3: {
                $$type: 'ClaimData3',
                // sourceNftContractAddress: beginCell().storeSlice(beginCell().storeStringTail("kQD7SUxVX7idrzk14ANs0oJPnDcCQ2kz4OAaWWZWJWTDifFd").endCell().asSlice()).endCell(),
                sourceNftContractAddress: beginCell().storeSlice(beginCell().storeAddress(Address.parseFriendly("kQD7SUxVX7idrzk14ANs0oJPnDcCQ2kz4OAaWWZWJWTDifFd").address).endCell().asSlice()).endCell(),
                fee: toNano("0.1"),
                royaltyReceiver: Address.parseFriendly("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh").address,
                metadata: 'https://s.getgems.io/nft-staging/c/628f6ab8077060a7a8d52d63/0.json',
            },
            data4: {
                $$type: 'ClaimData4',
                newContent: newContent,
                transactionHash: '0x6f7C0c6A6dd6E435b0EEc1c9F7Bce01A1908f386',
                royalty: {
                    $$type: 'RoyaltyParams',
                    numerator: 1000n,
                    denominator: 350n,
                    destination: contract.address
                }
            }
        }

        let signature = sign(beginCell().store(storeClaimData(claimData)).endCell().hash(), key.secretKey);

        let sig: SignerAndSignature = {
            $$type: 'SignerAndSignature',
            key: publicKey,
            signature: beginCell().storeBuffer(signature).endCell()
        };

        let dictA = Dictionary.empty<bigint, SignerAndSignature>().set(0n, sig);



        console.log(storeClaimData(claimData).toString());

        await contract.send(
            treasure,
            {
                value: toNano('1')
            }, {
            $$type: "ClaimNFT721",
            data: claimData,
            len: 1n,
            signatures: dictA
        });

        await system.run();

        expect(tracker.collect()).toMatchSnapshot();
        expect(storageDeployerTracker.collect()).toMatchSnapshot();
        expect(storageTracker.collect()).toMatchSnapshot();

        // expect(storagetracker?.collect()).toMatchSnapshot();
        // expect(await contract.getSeqno()).toBe(1n);

        // // Send empty message
        // await contract.send(treasure, { value: toNano(1) }, 'notify');
        // await system.run();
        // expect(tracker.collect()).toMatchSnapshot();
        // expect(await contract.getSeqno()).toBe(2n);

        // // Send comment message
        // await contract.send(treasure, { value: toNano(1) }, null);
        // await system.run();
        // expect(tracker.collect()).toMatchSnapshot();
        // expect(await contract.getSeqno()).toBe(3n);
    });
});