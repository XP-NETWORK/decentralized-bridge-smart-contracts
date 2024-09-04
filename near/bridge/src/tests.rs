
use near_sdk::AccountId;

use crate::Bridge;

use std::str::FromStr;

#[test]
fn initializes_correctly() {
    let cid = AccountId::from_str("aid").unwrap();
    let sid = AccountId::from_str("aid").unwrap();
    Bridge::new(cid, sid, vec![]);
}
