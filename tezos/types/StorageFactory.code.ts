
export const StorageFactoryCode: { __type: 'StorageFactoryCode', protocol: string, code: object[] } = {
    __type: 'StorageFactoryCode',
    protocol: 'PtEdo2ZkT9oKpimTah6x2embF25oss54njMuPzkJTEi5RqfdZFA',
    code: JSON.parse(`[{"prim":"parameter","args":[{"prim":"or","args":[{"prim":"address","annots":["%deploy_sft_storage"]},{"prim":"or","args":[{"prim":"address","annots":["%deploy_nft_storage"]},{"prim":"address","annots":["%set_owner"]}]}]}]},{"prim":"storage","args":[{"prim":"pair","args":[{"prim":"option","annots":["%owner"],"args":[{"prim":"address"}]},{"prim":"map","annots":["%collection_to_store"],"args":[{"prim":"address"},{"prim":"address"}]}]}]},{"prim":"code","args":[[{"prim":"LAMBDA","args":[{"prim":"pair","args":[{"prim":"option","args":[{"prim":"address"}]},{"prim":"map","args":[{"prim":"address"},{"prim":"address"}]}]},{"prim":"unit"},[{"prim":"DUP"},{"prim":"CAR"},{"prim":"IF_NONE","args":[[{"prim":"PUSH","args":[{"prim":"bool"},{"prim":"True"}]}],[{"prim":"DROP"},{"prim":"PUSH","args":[{"prim":"bool"},{"prim":"False"}]}]]},{"prim":"IF","args":[[{"prim":"PUSH","args":[{"prim":"string"},{"string":"Contract is not initialized"}]},{"prim":"FAILWITH"}],[]]},{"prim":"SENDER"},{"prim":"SWAP"},{"prim":"CAR"},{"prim":"IF_NONE","args":[[{"prim":"PUSH","args":[{"prim":"string"},{"string":"Unreachable"}]},{"prim":"FAILWITH"}],[]]},{"prim":"COMPARE"},{"prim":"NEQ"},{"prim":"IF","args":[[{"prim":"PUSH","args":[{"prim":"string"},{"string":"Only owner can call this function"}]},{"prim":"FAILWITH"}],[]]},{"prim":"UNIT"}]]},{"prim":"SWAP"},{"prim":"UNPAIR"},{"prim":"IF_LEFT","args":[[{"prim":"DUP","args":[{"int":"2"}]},{"prim":"DIG","args":[{"int":"3"}]},{"prim":"SWAP"},{"prim":"EXEC"},{"prim":"DROP"},{"prim":"DUP"},{"prim":"DUP","args":[{"int":"3"}]},{"prim":"CAR"},{"prim":"IF_NONE","args":[[{"prim":"PUSH","args":[{"prim":"string"},{"string":"Unreachable"}]},{"prim":"FAILWITH"}],[]]},{"prim":"PAIR"},{"prim":"PUSH","args":[{"prim":"mutez"},{"int":"0"}]},{"prim":"NONE","args":[{"prim":"key_hash"}]},{"prim":"CREATE_CONTRACT","args":[[{"prim":"parameter","args":[{"prim":"or","args":[{"prim":"pair","annots":["%unlock_token"],"args":[{"prim":"nat","annots":["%token_id"]},{"prim":"address","annots":["%to"]},{"prim":"nat","annots":["%amt"]}]},{"prim":"pair","annots":["%deposit_token"],"args":[{"prim":"nat","annots":["%token_id"]},{"prim":"nat","annots":["%amt"]}]}]}]},{"prim":"storage","args":[{"prim":"pair","args":[{"prim":"address","annots":["%owner"]},{"prim":"address","annots":["%collection"]}]}]},{"prim":"code","args":[[{"prim":"LAMBDA","args":[{"prim":"pair","args":[{"prim":"address"},{"prim":"address"},{"prim":"nat"},{"prim":"nat"},{"prim":"address"},{"prim":"address"}]},{"prim":"operation"},[{"prim":"UNPAIR","args":[{"int":"5"}]},{"prim":"DIG","args":[{"int":"4"}]},{"prim":"CDR"},{"prim":"CONTRACT","annots":["%transfer"],"args":[{"prim":"list","args":[{"prim":"pair","args":[{"prim":"address","annots":["%from_"]},{"prim":"list","annots":["%txs"],"args":[{"prim":"pair","args":[{"prim":"address","annots":["%to_"]},{"prim":"nat","annots":["%token_id"]},{"prim":"nat","annots":["%amount"]}]}]}]}]}]},{"prim":"IF_NONE","args":[[{"prim":"PUSH","args":[{"prim":"string"},{"string":"bad address for get_entrypoint"}]},{"prim":"FAILWITH"}],[]]},{"prim":"PUSH","args":[{"prim":"mutez"},{"int":"0"}]},{"prim":"NIL","args":[{"prim":"pair","args":[{"prim":"address"},{"prim":"list","args":[{"prim":"pair","args":[{"prim":"address"},{"prim":"nat"},{"prim":"nat"}]}]}]}]},{"prim":"NIL","args":[{"prim":"pair","args":[{"prim":"address"},{"prim":"nat"},{"prim":"nat"}]}]},{"prim":"DIG","args":[{"int":"7"}]},{"prim":"DIG","args":[{"int":"7"}]},{"prim":"DIG","args":[{"int":"7"}]},{"prim":"PAIR","args":[{"int":"3"}]},{"prim":"CONS"},{"prim":"DIG","args":[{"int":"4"}]},{"prim":"PAIR"},{"prim":"CONS"},{"prim":"TRANSFER_TOKENS"}]]},{"prim":"SWAP"},{"prim":"UNPAIR"},{"prim":"IF_LEFT","args":[[{"prim":"SELF_ADDRESS"},{"prim":"DUP","args":[{"int":"3"}]},{"prim":"CAR"},{"prim":"SENDER"},{"prim":"COMPARE"},{"prim":"NEQ"},{"prim":"IF","args":[[{"prim":"PUSH","args":[{"prim":"string"},{"string":"Cannot perform this function since you are not the owner."}]},{"prim":"FAILWITH"}],[]]},{"prim":"DUP","args":[{"int":"3"}]},{"prim":"DUP","args":[{"int":"3"}]},{"prim":"GET","args":[{"int":"4"}]},{"prim":"DUP","args":[{"int":"4"}]},{"prim":"CAR"},{"prim":"DIG","args":[{"int":"4"}]},{"prim":"GET","args":[{"int":"3"}]},{"prim":"DIG","args":[{"int":"4"}]},{"prim":"PAIR","args":[{"int":"5"}]},{"prim":"DIG","args":[{"int":"2"}]},{"prim":"SWAP"},{"prim":"EXEC"}],[{"prim":"SELF_ADDRESS"},{"prim":"DUP","args":[{"int":"3"}]},{"prim":"DUP","args":[{"int":"3"}]},{"prim":"CDR"},{"prim":"DIG","args":[{"int":"3"}]},{"prim":"CAR"},{"prim":"DIG","args":[{"int":"3"}]},{"prim":"SOURCE"},{"prim":"PAIR","args":[{"int":"5"}]},{"prim":"DIG","args":[{"int":"2"}]},{"prim":"SWAP"},{"prim":"EXEC"}]]},{"prim":"SWAP"},{"prim":"NIL","args":[{"prim":"operation"}]},{"prim":"DIG","args":[{"int":"2"}]},{"prim":"CONS"},{"prim":"PAIR"}]]},{"prim":"view","args":[{"string":"get_collection_address"},{"prim":"unit"},{"prim":"address"},[{"prim":"CDR"},{"prim":"CDR"}]]},{"prim":"view","args":[{"string":"get_owner"},{"prim":"unit"},{"prim":"address"},[{"prim":"CDR"},{"prim":"CDR"}]]}]]},{"prim":"PAIR"},{"prim":"DUP","args":[{"int":"3"}]},{"prim":"CDR"},{"prim":"DUP","args":[{"int":"2"}]},{"prim":"CDR"},{"prim":"DIG","args":[{"int":"3"}]},{"prim":"DUG","args":[{"int":"2"}]},{"prim":"SOME"},{"prim":"DIG","args":[{"int":"2"}]},{"prim":"UPDATE"},{"prim":"DIG","args":[{"int":"2"}]},{"prim":"SWAP"},{"prim":"UPDATE","args":[{"int":"2"}]},{"prim":"NIL","args":[{"prim":"operation"}]},{"prim":"DIG","args":[{"int":"2"}]},{"prim":"CAR"},{"prim":"CONS"}],[{"prim":"IF_LEFT","args":[[{"prim":"DUP","args":[{"int":"2"}]},{"prim":"DIG","args":[{"int":"3"}]},{"prim":"SWAP"},{"prim":"EXEC"},{"prim":"DROP"},{"prim":"EMPTY_SET","args":[{"prim":"nat"}]},{"prim":"DUP","args":[{"int":"2"}]},{"prim":"DUP","args":[{"int":"4"}]},{"prim":"CAR"},{"prim":"IF_NONE","args":[[{"prim":"PUSH","args":[{"prim":"string"},{"string":"Unreachable"}]},{"prim":"FAILWITH"}],[]]},{"prim":"PAIR","args":[{"int":"3"}]},{"prim":"PUSH","args":[{"prim":"mutez"},{"int":"0"}]},{"prim":"NONE","args":[{"prim":"key_hash"}]},{"prim":"CREATE_CONTRACT","args":[[{"prim":"parameter","args":[{"prim":"or","args":[{"prim":"pair","annots":["%unlock_token"],"args":[{"prim":"nat","annots":["%token_id"]},{"prim":"address","annots":["%to"]}]},{"prim":"nat","annots":["%add_deposited_token"]}]}]},{"prim":"storage","args":[{"prim":"pair","args":[{"prim":"address","annots":["%owner"]},{"prim":"address","annots":["%collection"]},{"prim":"set","annots":["%locked"],"args":[{"prim":"nat"}]}]}]},{"prim":"code","args":[[{"prim":"LAMBDA","args":[{"prim":"pair","args":[{"prim":"address"},{"prim":"address"},{"prim":"set","args":[{"prim":"nat"}]}]},{"prim":"unit"},[{"prim":"CAR"},{"prim":"SENDER"},{"prim":"COMPARE"},{"prim":"NEQ"},{"prim":"IF","args":[[{"prim":"PUSH","args":[{"prim":"string"},{"string":"Cannot perform this function since you are not the owner."}]},{"prim":"FAILWITH"}],[]]},{"prim":"UNIT"}]]},{"prim":"SWAP"},{"prim":"UNPAIR"},{"prim":"IF_LEFT","args":[[{"prim":"SELF_ADDRESS"},{"prim":"DUP","args":[{"int":"3"}]},{"prim":"DIG","args":[{"int":"4"}]},{"prim":"SWAP"},{"prim":"EXEC"},{"prim":"DROP"},{"prim":"DUP","args":[{"int":"3"}]},{"prim":"GET","args":[{"int":"3"}]},{"prim":"CONTRACT","annots":["%transfer"],"args":[{"prim":"list","args":[{"prim":"pair","args":[{"prim":"address","annots":["%from_"]},{"prim":"list","annots":["%txs"],"args":[{"prim":"pair","args":[{"prim":"address","annots":["%to_"]},{"prim":"nat","annots":["%token_id"]},{"prim":"nat","annots":["%amount"]}]}]}]}]}]},{"prim":"IF_NONE","args":[[{"prim":"PUSH","args":[{"prim":"string"},{"string":"bad address for get_entrypoint"}]},{"prim":"FAILWITH"}],[]]},{"prim":"PUSH","args":[{"prim":"mutez"},{"int":"0"}]},{"prim":"NIL","args":[{"prim":"pair","args":[{"prim":"address"},{"prim":"list","args":[{"prim":"pair","args":[{"prim":"address"},{"prim":"nat"},{"prim":"nat"}]}]}]}]},{"prim":"NIL","args":[{"prim":"pair","args":[{"prim":"address"},{"prim":"nat"},{"prim":"nat"}]}]},{"prim":"PUSH","args":[{"prim":"nat"},{"int":"1"}]},{"prim":"DUP","args":[{"int":"7"}]},{"prim":"CAR"},{"prim":"DUP","args":[{"int":"8"}]},{"prim":"CDR"},{"prim":"PAIR","args":[{"int":"3"}]},{"prim":"CONS"},{"prim":"DIG","args":[{"int":"4"}]},{"prim":"PAIR"},{"prim":"CONS"},{"prim":"TRANSFER_TOKENS"},{"prim":"DUP","args":[{"int":"3"}]},{"prim":"DIG","args":[{"int":"3"}]},{"prim":"GET","args":[{"int":"4"}]},{"prim":"PUSH","args":[{"prim":"bool"},{"prim":"False"}]},{"prim":"DIG","args":[{"int":"4"}]},{"prim":"CAR"},{"prim":"UPDATE"},{"prim":"UPDATE","args":[{"int":"4"}]},{"prim":"NIL","args":[{"prim":"operation"}]},{"prim":"DIG","args":[{"int":"2"}]},{"prim":"CONS"}],[{"prim":"DUP","args":[{"int":"2"}]},{"prim":"DIG","args":[{"int":"3"}]},{"prim":"SWAP"},{"prim":"EXEC"},{"prim":"DROP"},{"prim":"DUP","args":[{"int":"2"}]},{"prim":"DIG","args":[{"int":"2"}]},{"prim":"GET","args":[{"int":"4"}]},{"prim":"PUSH","args":[{"prim":"bool"},{"prim":"True"}]},{"prim":"DIG","args":[{"int":"3"}]},{"prim":"UPDATE"},{"prim":"UPDATE","args":[{"int":"4"}]},{"prim":"NIL","args":[{"prim":"operation"}]}]]},{"prim":"PAIR"}]]},{"prim":"view","args":[{"string":"get_collection_address"},{"prim":"unit"},{"prim":"address"},[{"prim":"CDR"},{"prim":"GET","args":[{"int":"3"}]}]]},{"prim":"view","args":[{"string":"has_locked_token"},{"prim":"nat"},{"prim":"bool"},[{"prim":"UNPAIR"},{"prim":"SWAP"},{"prim":"GET","args":[{"int":"4"}]},{"prim":"SWAP"},{"prim":"MEM"}]]},{"prim":"view","args":[{"string":"get_owner"},{"prim":"unit"},{"prim":"address"},[{"prim":"CDR"},{"prim":"GET","args":[{"int":"3"}]}]]}]]},{"prim":"PAIR"},{"prim":"DUP","args":[{"int":"3"}]},{"prim":"CDR"},{"prim":"DUP","args":[{"int":"2"}]},{"prim":"CDR"},{"prim":"DIG","args":[{"int":"3"}]},{"prim":"DUG","args":[{"int":"2"}]},{"prim":"SOME"},{"prim":"DIG","args":[{"int":"2"}]},{"prim":"UPDATE"},{"prim":"DIG","args":[{"int":"2"}]},{"prim":"SWAP"},{"prim":"UPDATE","args":[{"int":"2"}]},{"prim":"NIL","args":[{"prim":"operation"}]},{"prim":"DIG","args":[{"int":"2"}]},{"prim":"CAR"},{"prim":"CONS"}],[{"prim":"DIG","args":[{"int":"2"}]},{"prim":"DROP"},{"prim":"DUP","args":[{"int":"2"}]},{"prim":"CAR"},{"prim":"IF_NONE","args":[[{"prim":"PUSH","args":[{"prim":"bool"},{"prim":"False"}]}],[{"prim":"DROP"},{"prim":"PUSH","args":[{"prim":"bool"},{"prim":"True"}]}]]},{"prim":"IF","args":[[{"prim":"PUSH","args":[{"prim":"string"},{"string":"Owner already set"}]},{"prim":"FAILWITH"}],[]]},{"prim":"SOME"},{"prim":"UPDATE","args":[{"int":"1"}]},{"prim":"NIL","args":[{"prim":"operation"}]}]]}]]},{"prim":"PAIR"}]]},{"prim":"view","args":[{"string":"get_store"},{"prim":"address"},{"prim":"address"},[{"prim":"UNPAIR"},{"prim":"SWAP"},{"prim":"CDR"},{"prim":"SWAP"},{"prim":"GET"},{"prim":"IF_NONE","args":[[{"prim":"PUSH","args":[{"prim":"string"},{"string":"MAP FIND"}]},{"prim":"FAILWITH"}],[]]}]]},{"prim":"view","args":[{"string":"get_owner"},{"prim":"unit"},{"prim":"address"},[{"prim":"CDR"},{"prim":"CAR"},{"prim":"IF_NONE","args":[[{"prim":"PUSH","args":[{"prim":"string"},{"string":"option is None"}]},{"prim":"FAILWITH"}],[]]}]]}]`)
};
