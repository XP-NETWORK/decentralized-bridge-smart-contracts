import { Wallet } from "ethers";
import { ethers } from "hardhat"

(async () => {

    const receiver = "0xe7D463DFf4E8c01040DafD137598d006292A7Aa3";
    const privateKeys = [
        "0xcaa224780e61681bee0aa83c81385822051b2b3862680a815305d05d67bcce26",
        "0x4554a15ded14f6e9abe7ad24e62307d6b745d2390561eb68012d2b7a11ab159f",
        "0x246d8b7a13d9b3004d5177683120c7ccd581551f29026eb0a96b5e356a58892b"
    ];

    for await (const x of privateKeys) {
        const wallet = new Wallet(x, ethers.provider);
        const balance = await ethers.provider.getBalance(wallet.address);
        console.log("balance",wallet.address, Number(balance) / 1e18);

        // Estimate gas limit
        continue
        const gasPrice = (await ethers.provider.getFeeData()).gasPrice || BigInt(0);

        console.log("gasPrice",gasPrice);
        
        const gasLimit = await ethers.provider.estimateGas({
            to: receiver,
            from: wallet.address,
            gasPrice: gasPrice,
        });

        console.log("gasLimit",gasLimit);


        // Calculate the maximum amount to send (subtract gas fee)
        const maxAmount = balance - (gasPrice * BigInt(21000));

        // Create and send the transaction
        const tx = {
            to: receiver,
            value: maxAmount,
            gasLimit: gasLimit,
            gasPrice: gasPrice,
        };
        if (balance > 0) {
            let rese = await wallet.sendTransaction(tx);
            console.log("txHash", (await rese.getTransaction())?.hash);
        }
    }
})();