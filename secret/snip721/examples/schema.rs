use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use snip721::msg::{
    Snip721ExecuteAnswer, Snip721QueryAnswer, Snip721QueryMsg, Snip721ExecuteMsg, Snip721InstantiateMsg,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(Snip721InstantiateMsg), &out_dir);
    export_schema(&schema_for!(Snip721ExecuteMsg), &out_dir);
    export_schema(&schema_for!(Snip721ExecuteAnswer), &out_dir);
    export_schema(&schema_for!(Snip721QueryMsg), &out_dir);
    export_schema(&schema_for!(Snip721QueryAnswer), &out_dir);
}
