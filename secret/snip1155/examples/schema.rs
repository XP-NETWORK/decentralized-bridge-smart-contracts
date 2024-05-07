use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use snip1155::msg::{Snip1155InstantiateMsg, Snip1155ExecuteMsg, Snip1155ExecuteAnswer, Snip1155QueryMsg, Snip1155QueryAnswer};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(Snip1155InstantiateMsg), &out_dir);
    export_schema(&schema_for!(Snip1155ExecuteMsg), &out_dir);
    export_schema(&schema_for!(Snip1155ExecuteAnswer), &out_dir);
    export_schema(&schema_for!(Snip1155QueryMsg), &out_dir);
    export_schema(&schema_for!(Snip1155QueryAnswer), &out_dir);
}
