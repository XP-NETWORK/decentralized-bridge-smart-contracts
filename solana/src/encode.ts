import { field, fixedArray, option, vec } from "@dao-xyz/borsh";
import { Struct, PublicKey } from "@solana/web3.js";
import { BN } from "@project-serum/anchor";

export class NftData {
    @field({
        serialize(arg: PublicKey, writer) {
            writer.writeFixedArray(arg.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    creatorKey: PublicKey;
    @field({ type: "String" })
    uri: string;
    @field({ type: "String" })
    title: string;

    constructor(args: { creatorKey: PublicKey; uri: string; title: string }) {
        this.creatorKey = args.creatorKey;
        this.uri = args.uri;
        this.title = args.title;
    }
}

export class InitializeData {
    @field({
        serialize(arg: PublicKey, writer) {
            writer.writeFixedArray(arg.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    validatorPublicKey: PublicKey;

    constructor(args: { validatorPublicKey: PublicKey }) {
        this.validatorPublicKey = args.validatorPublicKey;
    }
}

export class NewValidatorPublicKey {
    @field({
        serialize(arg: PublicKey, writer) {
            writer.writeFixedArray(arg.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    validatorPublicKey: PublicKey;

    constructor(args: { validatorPublicKey: PublicKey }) {
        this.validatorPublicKey = args.validatorPublicKey;
    }
}

export class SignatureInfo {
    @field({
        serialize(arg: PublicKey, writer) {
            writer.writeFixedArray(arg.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    publicKey: PublicKey;
    @field({ type: fixedArray("u8", 64) })
    sig: number[];

    constructor(args: { publicKey: PublicKey; sig: number[] }) {
        this.publicKey = args.publicKey;
        this.sig = args.sig;
    }
}

export class AddValidatorData {
    @field({
        serialize(arg: PublicKey, writer) {
            writer.writeFixedArray(arg.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    validatorPublicKey: PublicKey;

    constructor(args: { validatorPublicKey: PublicKey }) {
        this.validatorPublicKey = args.validatorPublicKey;
    }
}

export class VerifyAddValidatorSignaturesData {
    @field({
        serialize(arg: PublicKey, writer) {
            writer.writeFixedArray(arg.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    validatorPublicKey: PublicKey;
    @field({ type: vec(SignatureInfo) })
    signatures: SignatureInfo[];

    constructor(args: { validatorPublicKey: PublicKey; signatures: SignatureInfo[] }) {
        this.validatorPublicKey = args.validatorPublicKey;
        this.signatures = args.signatures;
    }
}

export class ClaimNftData {
    @field({ type: "String" })
    tokenId: string;

    @field({ type: "String" })
    sourceChain: string;

    @field({ type: "String" })
    destinationChain: string;

    @field({
        serialize(arg: PublicKey, writer) {
            writer.writeFixedArray(arg.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    destinationUserAddress: PublicKey;

    @field({ type: "String" })
    sourceNftContractAddress: string;

    @field({ type: "String" })
    name: string;

    @field({ type: "String" })
    symbol: string;

    @field({ type: "u64" })
    royalty: BN;

    @field({
        serialize(arg: PublicKey, writer) {
            writer.writeFixedArray(arg.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    royaltyReceiver: PublicKey;

    @field({ type: "String" })
    metadata: string;

    @field({ type: "String" })
    transactionHash: string;

    @field({ type: "u64" })
    tokenAmount: BN;

    @field({ type: "String" })
    nftType: string;

    @field({ type: "u64" })
    fee: BN;

    constructor(args: {
        tokenId: string;
        sourceChain: string;
        destinationChain: string;
        destinationUserAddress: PublicKey;
        sourceNftContractAddress: string;
        name: string;
        symbol: string;
        royalty: BN;
        royaltyReceiver: PublicKey;
        metadata: string;
        transactionHash: string;
        tokenAmount: BN;
        nftType: string;
        fee: BN;
    }) {
        this.tokenId = args.tokenId;
        this.sourceChain = args.sourceChain;
        this.destinationChain = args.destinationChain;
        this.destinationUserAddress = args.destinationUserAddress;
        this.sourceNftContractAddress = args.sourceNftContractAddress;
        this.name = args.name;
        this.symbol = args.symbol;
        this.royalty = args.royalty;
        this.royaltyReceiver = args.royaltyReceiver;
        this.metadata = args.metadata;
        this.transactionHash = args.transactionHash;
        this.tokenAmount = args.tokenAmount;
        this.nftType = args.nftType;
        this.fee = args.fee;
    }
}

export class ClaimData {
    @field({ type: ClaimNftData })
    claimData: ClaimNftData;
    // @field({ type: vec(SignatureInfo) })
    // signatures: SignatureInfo[];
    // @field({ type: "u8" })
    // bridgeBump: number;
    // @field({ type: "u8" })
    // otherTokensBump: number;
    // @field({ type: "u8" })
    // selfTokensBump: number;
    // @field({ type: "u8" })
    // collectionBump: number;
    @field({
        type: option(PublicKey),
        serialize(arg: PublicKey, writer) {
            writer.writeFixedArray(arg.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    nftMint: PublicKey | undefined;

    constructor(args:
        {
            claimData: ClaimNftData,
            nftMint: PublicKey,
            // signatures: SignatureInfo[],
            // bridgeBump: number,
            // otherTokensBump: number,
            // selfTokensBump: number,
            // collectionBump: number,
            // collectionMintKey: PublicKey
        }) {
        this.claimData = args.claimData;
        this.nftMint = args.nftMint;
        // this.signatures = args.signatures;
        // this.bridgeBump = args.bridgeBump;
        // this.otherTokensBump = args.otherTokensBump;
        // this.selfTokensBump = args.selfTokensBump;
        // this.collectionBump = args.collectionBump;
        // this.collectionMintKey = args.collectionMintKey;
    }
}

export class LockData {
    @field({
        serialize(arg: PublicKey, writer) {
            writer.writeFixedArray(arg.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    tokenId: PublicKey;
    @field({ type: "String" })
    destinationChain: string;
    @field({ type: "String" })
    destinationUserAddress: string;
    @field({
        serialize(arg: PublicKey, writer) {
            writer.writeFixedArray(arg.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    sourceNftContractAddress: PublicKey;
    @field({ type: "u64" })
    tokenAmount: BN;
    @field({ type: "u8" })
    bridgeBump: number;
    @field({ type: "u8" })
    otherTokensBump: number;
    @field({ type: "u8" })
    selfTokensBump: number;

    constructor(args:
        {
            tokenId: PublicKey,
            destinationChain: string,
            destinationUserAddress: string,
            sourceNftContractAddress: PublicKey,
            tokenAmount: BN,
            bridgeBump: number,
            otherTokensBump: number,
            selfTokensBump: number,
        }) {
        this.tokenId = args.tokenId;
        this.destinationChain = args.destinationChain;
        this.destinationUserAddress = args.destinationUserAddress;
        this.sourceNftContractAddress = args.sourceNftContractAddress;
        this.tokenAmount = args.tokenAmount;
        this.bridgeBump = args.bridgeBump;
        this.otherTokensBump = args.otherTokensBump;
        this.selfTokensBump = args.selfTokensBump;
    }
}

export class TxHash {
    @field({ type: fixedArray("u8", 32) })
    txHash: number[];

    constructor(args: { txHash: number[] }) {
        this.txHash = args.txHash;
    }
}

export class VerifyClaimSignaturesData {
    @field({ type: fixedArray("u8", 32) })
    txHash: number[];
    @field({ type: ClaimNftData })
    claimData: ClaimNftData;
    @field({ type: vec(SignatureInfo) })
    signatures: SignatureInfo[];

    constructor(args: { txHash: number[]; claimData: ClaimNftData; signatures: SignatureInfo[] }) {
        this.txHash = args.txHash;
        this.claimData = args.claimData;
        this.signatures = args.signatures;
    }
}