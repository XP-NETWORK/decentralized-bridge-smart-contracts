import { Address, Dictionary, beginCell, internal, toNano } from '@ton/core';
import { Bridge, ClaimData, SignerAndSignature, storeClaimData } from '../wrappers/Bridge';
import { NetworkProvider } from '@ton/blueprint';
import { testKey } from '@tact-lang/emulator';
import { NFTCollectionDeployer } from '../build/Bridge/tact_NFTCollectionDeployer';
import { NftCollection } from '../build/Bridge/tact_NftCollection';
import { sign } from 'ton-crypto';
import { NftItem } from '../build/Bridge/tact_NftItem';
import { mnemonicNew, mnemonicToPrivateKey } from '@ton/crypto';
import { TonClient, } from '@ton/ton/dist/client/TonClient';

export async function run(provider: NetworkProvider) {

    let key = testKey('dfdsfdssssffffggg');
    let publicKey = beginCell().storeBuffer(Buffer.from("5aba2a59ffcef4fc7894ac0682dd419b18a54c30b55d0db0e244f15b1a7f87b2", "hex")).endCell().beginParse().loadUintBig(256);

    // let nftItem = provider.open(NftItem.fromAddress(Address.parseFriendly("EQC9mmKMKCnoMqmaJiuUJPe2WL3V0j23mzZBvkkve_eAjOh5").address));
    // console.log((await nftItem.getGetNftData()).individual_content.asSlice().loadStringTail());

    const bridge = provider.open(Bridge.fromAddress(Address.parseFriendly("EQDI6P9gheuWLh1euThjFE2muUpa9tp2y49TD6Zz5oOF5gWL").address));

    console.log("collection deployer ", await bridge.getCollectionDeployer() + "\n");
    console.log("storage deployer ", await bridge.getStorageDeployer()) + "\n";

    console.log("token info ", await bridge.getTokenInfo(Address.parseFriendly("EQALVS0Pz2gYa6-HoNBsd8XJeu3Afskz8LRfsb1M7XujNtR9").address));
    console.log("token info self", await bridge.getTokenInfoSelf(0n, "BSC", beginCell().storeSlice(beginCell().storeStringTail("0x491d6f9f14e0cd58d5094333ae172cdd19c87781").endCell().asSlice()).endCell()));
    // console.log("token info self", await bridge.getTokenInfoSelf(0n, "TON", beginCell().storeSlice(beginCell().storeAddress(Address.parseFriendly("EQALVS0Pz2gYa6-HoNBsd8XJeu3Afskz8LRfsb1M7XujNtR9").address).endCell().asSlice()).endCell()));



    console.log("original 721 ", await bridge.getOriginal721Mapping(Address.parseFriendly("EQALVS0Pz2gYa6-HoNBsd8XJeu3Afskz8LRfsb1M7XujNtR9").address, "TON") + "\n");
    console.log("duplicate 721 ", await bridge.getDuplicate721Mapping(Address.parseFriendly("EQALVS0Pz2gYa6-HoNBsd8XJeu3Afskz8LRfsb1M7XujNtR9").address) + "\n");


    console.log("original to duplicate ", await bridge.getOriginalToDuplicate("EQALVS0Pz2gYa6-HoNBsd8XJeu3Afskz8LRfsb1M7XujNtR9", "TON") + "\n");
    console.log("duplicate to original ", await bridge.getDuplicateToOriginal(Address.parseFriendly("EQALVS0Pz2gYa6-HoNBsd8XJeu3Afskz8LRfsb1M7XujNtR9").address) + "\n");

}
