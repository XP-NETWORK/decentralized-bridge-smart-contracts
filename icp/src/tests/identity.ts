import {Ed25519KeyIdentity} from "@dfinity/identity"

const key = Ed25519KeyIdentity.fromSecretKey(Buffer.from("efeb1c2fde35ba8cada52300388a0455f5f75e20bd15efdf3a2b29af172a3379", "hex"))

console.log(key.getPrincipal())