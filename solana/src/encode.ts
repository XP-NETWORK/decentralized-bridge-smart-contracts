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

    constructor(args: { creatorKey: PublicKey, uri: string, title: string }) {
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
    publicKey: PublicKey;

    constructor(args: { publicKey: PublicKey }) {
        this.publicKey = args.publicKey;
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
    publicKey: PublicKey;

    constructor(args: { publicKey: PublicKey }) {
        this.publicKey = args.publicKey;
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

    constructor(args: { publicKey: PublicKey, sig: number[] }) {
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
    publicKey: PublicKey;
    @field({ type: vec(SignatureInfo) })
    signatures: SignatureInfo[]

    constructor(args: { publicKey: PublicKey, signatures: SignatureInfo[] }) {
        this.publicKey = args.publicKey;
        this.signatures = args.signatures;
    }
}

export class PauseData {
    @field({ type: "u64" })
    actionId: BN;
    @field({ type: "u8" })
    bridgeBump: number;

    constructor(args: { actionId: BN; bridgeBump: number }) {
        this.actionId = args.actionId;
        this.bridgeBump = args.bridgeBump;
    }
}

export class UnpauseData {
    @field({ type: "u64" })
    actionId: BN;
    @field({ type: "u8" })
    bridgeBump: number;

    constructor(args: { actionId: BN, bridgeBump: number }) {
        this.actionId = args.actionId;
        this.bridgeBump = args.bridgeBump;
    }
}

export class UpdateGroupkeyData {
    @field({ type: "u64" })
    actionId: BN;
    @field({ type: "u8" })
    bridgeBump: number;
    @field({ type: fixedArray("u8", 32) })
    newKey: number[];

    constructor(args: { actionId: BN, bridgeBump: number, newKey: number[] }) {
        this.actionId = args.actionId;
        this.bridgeBump = args.bridgeBump;
        this.newKey = args.newKey;
    }
}

export class Collection {
    @field({ type: "bool" })
    verified: boolean;
    @field({
        serialize(address: PublicKey, writer) {
            writer.writeFixedArray(address.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    key: PublicKey;

    constructor(args: { verified: boolean, key: PublicKey }) {
        this.verified = args.verified;
        this.key = args.key;
    }
}

export class Creator {
    @field({
        serialize(arg: PublicKey, writer) {
            writer.writeFixedArray(arg.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    address: PublicKey;
    @field({ type: "bool" })
    verified: boolean;
    @field({ type: "u8" })
    share: number;

    constructor(args: { address: PublicKey, verified: boolean, share: number }) {
        this.address = args.address;
        this.verified = args.verified;
        this.share = args.share;
    }
}

export class TransferNftData {
    @field({ type: "u64" })
    actionId: BN;
    @field({ type: "u8" })
    bridgeBump: number;
    @field({ type: "u8" })
    authBump: number;
    @field({ type: "u64" })
    chainNonce: BN;
    @field({ type: "String" })
    name: string;
    @field({ type: "String" })
    symbol: string;
    @field({ type: "String" })
    uri: string;
    @field({
        serialize(arg: PublicKey, writer) {
            writer.writeFixedArray(arg.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    owner: PublicKey;
    @field({ type: option(Collection) })
    collection: Collection | undefined;
    @field({ type: option("u16") })
    sellerFeeBasisPoints: number | undefined;
    @field({ type: option(vec(Creator)) })
    creators: Creator[] | undefined;

    constructor(args: { actionId: BN; bridgeBump: number; authBump: number; chainNonce: BN; name: string; symbol: string; uri: string; owner: PublicKey; collection: Collection; sellerFeeBasisPoints: number; creators: Creator[] }) {
        this.actionId = args.actionId;
        this.bridgeBump = args.bridgeBump;
        this.authBump = args.authBump;
        this.chainNonce = args.chainNonce;
        this.name = args.name;
        this.symbol = args.symbol;
        this.uri = args.uri;
        this.owner = args.owner;
        this.collection = args.collection;
        this.sellerFeeBasisPoints = args.sellerFeeBasisPoints;
        this.creators = args.creators;
    }
}

export class UnfreezeNftData {
    @field({ type: "u64" })
    actionId: BN;
    @field({ type: "u8" })
    bridgeBump: number;
    @field({
        serialize(arg: PublicKey, writer) {
            writer.writeFixedArray(arg.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    receiver: PublicKey;
    @field({
        serialize(arg: PublicKey, writer) {
            writer.writeFixedArray(arg.toBuffer());
        },
        deserialize(reader) {
            return reader.readFixedArray(32);
        },
    })
    mint: PublicKey;

    constructor(args: { actionId: BN; bridgeBump: number; receiver: PublicKey; mint: PublicKey }) {
        this.actionId = args.actionId;
        this.bridgeBump = args.bridgeBump;
        this.receiver = args.receiver;
        this.mint = args.mint;
    }
}

export class WithdrawFeesData {
    @field({ type: "u64" })
    actionId: BN;
    @field({ type: "u8" })
    bridgeBump: number;
    constructor(args: { actionId: BN; bridgeBump: number }) {
        this.actionId = args.actionId;
        this.bridgeBump = args.bridgeBump;
    }
}
