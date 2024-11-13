use alloc::string::{String, ToString};
use casper_event_standard::Event;
use casper_types::{
    bytesrepr::{FromBytes, ToBytes}, CLType, CLTyped, PublicKey, U512
};

#[derive(Clone, Event, Debug)]
pub struct AddNewValidator {
    pub public_key: PublicKey,
}
