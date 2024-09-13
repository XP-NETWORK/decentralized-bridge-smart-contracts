import { KeyPair } from "near-api-js";

const kp = KeyPair.fromString(process.env.SK! as any)
console.log(Buffer.from(kp.getPublicKey().data).toString("hex"));