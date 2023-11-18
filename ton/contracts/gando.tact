import "@stdlib/deploy";
message Gnado {
    address: Address;
    chain: String;
}

contract Gando {
    storageMapping721: map<Int, Address>;
    init(msg: Gnado){
        let hash: Int = msg.toCell().hash();
        self.storageMapping721.set(hash, address("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh"));
    }

    receive(msg: Gnado){
        let hash: Int = msg.toCell().hash();
        let storageAddress: Address = self.storageMapping721.get(hash)!!;
        require(storageAddress == address("EQAV8tH2WDuWYU7zAmkJmIwP8Ph_uIC4zBqJNIfKgRUUQewh"), "GANDO HA");
    }
}