import { ContractSystem, testKey } from '@tact-lang/emulator';
import { Bridge } from '../wrappers/Bridge';
import { Address, beginCell, toNano } from 'ton-core';

(async () => {
    //
    // Init System
    //

    // Contract System is a virtual environment that emulates the TON blockchain
    const system = await ContractSystem.create();

    // Treasure is a contract that has 1m of TONs and is a handy entry point for your smart contracts
    let treasure = await system.treasure('name of treasure');

    //
    // Open contract
    //
    let key = testKey('wallet-keysss');
    console.log("key", key.publicKey.toString("hex"));
    console.log("key", key.secretKey.toString("hex"));
    let publicKey = beginCell().storeBuffer(key.publicKey).endCell().beginParse().loadUintBig(256);
    // Contract itself
    let contract = system.open(await Bridge.fromInit(publicKey, "TON"));

    // This object would track all transactions in this contract
    let tracker = system.track(contract.address);

    // This object would track all logs
    let logger = system.log(contract.address);

    //
    // Sending a message
    //

    // First we enqueue a messages. NOTE: You can enqueue multiple messages in row
    await contract.send(treasure, { value: toNano(1) }, "Deploy");
    // await contract.send(treasure, { value: toNano(1) }, { $$type: "Increment" });
    // Run system until there are no more messages

    await contract.send(treasure, { value: toNano(1) }, "Lock721");
    await system.run();

    //
    // Collecting results
    //

    console.log(tracker.collect()[1].events[2]); // Prints out all transactions in contract
    
    // console.log(logger.collect()); // Prints out all logs for each transaction

    //
    // Invoking get methods
    //

    console.log(contract.address);
    console.log(await contract.getCollectionDeployer());
    console.log(await contract.getStorageDeployer());


})();





// import { ContractSystem, testKey } from '@tact-lang/emulator';
// import { Bridge } from '../wrappers/Bridge';
// import { beginCell, toNano } from 'ton-core';
// import { TodoParent } from '../build/Bridge/tact_TodoParent';

// (async () => {
//     //
//     // Init System
//     //

//     // Contract System is a virtual environment that emulates the TON blockchain
//     const system = await ContractSystem.create();

//     // Treasure is a contract that has 1m of TONs and is a handy entry point for your smart contracts
//     let treasure = await system.treasure('name of treasure');

//     //
//     // Open contract
//     //
//     let key = testKey('wallet-keysss');
//     console.log("key", key.publicKey.toString("hex"));
//     console.log("key", key.secretKey.toString("hex"));
//     let publicKey = beginCell().storeBuffer(key.publicKey).endCell().beginParse().loadUintBig(256);
//     // Contract itself
//     let contract = system.open(await TodoParent.fromInit());

//     // This object would track all transactions in this contract
//     let tracker = system.track(contract.address);

//     // This object would track all logs
//     let logger = system.log(contract.address);


//     //
//     // Sending a message
//     //

//     // First we enqueue a messages. NOTE: You can enqueue multiple messages in row
//     await contract.send(treasure, { value: toNano(1) }, "greet 3");
//     // await contract.send(treasure, { value: toNano(1) }, { $$type: "Increment" });
//     // Run system until there are no more messages
//     await system.run();

//     //
//     // Collecting results
//     //

//     console.log(tracker.collect()); // Prints out all transactions in contract
//     // console.log(logger.collect()); // Prints out all logs for each transaction

//     //
//     // Invoking get methods
//     //

//     console.log(contract.address);
//     // console.log(await contract.getCollectionDeployer());
//     // console.log(await contract.getStorageDeployer());


// })();
