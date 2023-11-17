import "@stdlib/deploy";
message Gnado {
    address: Address;
    chain: String;
}

contract Gando {
    storageMapping721: map<Int, Address>;
    init(){
        self.storageMapping721.set(4105371466, address("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh"));
    }

    receive(msg: Gnado){
        let slice: Slice = beginCell().storeAddress(msg.address).storeSlice(msg.chain.asSlice()).endCell().beginParse();
        let key: Int = slice.loadUint(32);
        let storageAddress: Address = self.storageMapping721.get(key)!!;
        require(storageAddress == address("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh"), "GANDO HA");
    }
}