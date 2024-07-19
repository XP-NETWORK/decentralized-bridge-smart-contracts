import * as web3 from "@solana/web3.js";
import * as fs from "fs";
(async () => {

    // connect to a cluster and get the current `slot`
    const url = process.env.ANCHOR_PROVIDER_URL;
    const options = anchor.AnchorProvider.defaultOptions();
    const connection = new web3.Connection(url, options.commitment);
    const slot = await connection.getSlot();
    const payer = web3.Keypair.fromSecretKey(
        Buffer.from(
            JSON.parse(
                fs.readFileSync(process.env.ANCHOR_WALLET, {
                    encoding: "utf-8",
                })
            )
        )
    );
    const wallet = new anchor.Wallet(payer);
    // Assumption:
    // `payer` is a valid `Keypair` with enough SOL to pay for the execution

    const [lookupTableInst, lookupTableAddress] =
        web3.AddressLookupTableProgram.createLookupTable({
            authority: payer.publicKey,
            payer: payer.publicKey,
            recentSlot: slot,
        });

    console.log("lookup table address:", lookupTableAddress.toBase58());

    // To create the Address Lookup Table on chain:
    // send the `lookupTableInst` instruction in a transaction
})();