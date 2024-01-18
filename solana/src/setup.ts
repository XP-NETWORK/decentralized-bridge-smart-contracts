import * as anchor from "@project-serum/anchor";
import fs from "fs/promises";
import { config } from "dotenv";
import { IDL } from "../target/types/xp_bridge";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";

config();

(async () => {
    const connection = new anchor.web3.Connection(
        process.env.ANCHOR_PROVIDER_URL!,
        "processed"
    );

    const gk = Buffer.from(process.env.SETUP_GROUP_KEY!, "hex");

    const file = JSON.parse(await fs.readFile(process.env.ANCHOR_WALLET, "utf-8"))

    const payer = Keypair.fromSecretKey(Buffer.from(file));

    const provider = new anchor.AnchorProvider(
        connection,
        new anchor.Wallet(payer),
        {}
    );

    const program = new anchor.Program(
        IDL,
        process.env.PROGRAM_ID!,
        provider
    );

    const encode = anchor.utils.bytes.utf8.encode;
    const [bridge] = await PublicKey.findProgramAddress(
        [encode("bridge")],
        program.programId
    );

    const response = await program.methods
        .initialize([...gk])
        .accounts({
            bridge: bridge,
            user: provider.wallet.publicKey,
            systemProgram: SystemProgram.programId,
        })
        .rpc();

    console.log(`Response: `, response);
})();
