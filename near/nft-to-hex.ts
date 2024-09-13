import fs from "fs"

const file = fs.readFileSync(__dirname + "/target/near/nft/nft.wasm");

const hex = file.toString("hex")

fs.writeFileSync(
  __dirname + "/nft.hex",
  `export const NFT_HEX = '${hex}'`
);