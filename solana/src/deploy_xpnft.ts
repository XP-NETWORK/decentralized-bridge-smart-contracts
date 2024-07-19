import {
    Metaplex,
    keypairIdentity,
    bundlrStorage,
} from "@metaplex-foundation/js";
import { Wallet } from "@project-serum/anchor";
import {
    Connection,
    clusterApiUrl,
    Keypair,
    LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { config } from "dotenv";
import fs from "fs/promises";

config();

(async () => {
    const connection = new Connection(process.env.ANCHOR_PROVIDER_URL!);

    const file = JSON.parse(
        await fs.readFile(process.env.ANCHOR_WALLET, "utf-8")
    );

    const payer = Keypair.fromSecretKey(Buffer.from(file));

    const programKpBuffer = JSON.parse(
        await fs.readFile(process.env.PROGRAM_WALLET, "utf-8")
    );

    const programKp = Keypair.fromSecretKey(Buffer.from(programKpBuffer));

    const metaplex = Metaplex.make(connection)
        .use(keypairIdentity(payer))

    const nftc = metaplex.nfts();

    const collection = await nftc.create(
        {
            name: "Xp Wrapped NFT",
            symbol: "XPNFT",
            uri: "ipfs://bafkreietvwrmdlvqqrolrne2hzkpg5pw5lp2g3yitcs3xexyevh3x6l6vu",
            isCollection: true,
            sellerFeeBasisPoints: 0,
            mintAuthority: programKp,
            collectionAuthority: programKp,
            creators: [
                {
                    address: programKp.publicKey,
                    share: 100,
                    authority: programKp
                }
            ],
            tokenOwner: programKp.publicKey,
            updateAuthority: programKp


        },
        {
            commitment: "processed",
        }
    );

    console.log(collection);
})();
