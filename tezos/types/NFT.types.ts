
import { ContractAbstractionFromContractType, WalletContractAbstractionFromContractType } from './type-utils';
import { address, BigMap, bytes, contract, Instruction, MMap, nat, ticket } from './type-aliases';

export type Storage = {
    metadata: BigMap<string, bytes>;
    assets: BigMap<nat, address>;
    token_metadata: BigMap<nat, {
        token_id: nat;
        token_info: MMap<string, bytes>;
    }>;
    operators: {Some: BigMap<{
        0: address;
        1: nat;
    }, Array<address>>} | null;
    approvals: BigMap<{
        0: address;
        1: address;
        2: nat;
    }, nat>;
    proxy: address;
    extension: address;
};

type Methods = {
    import_ticket: (param: Array<{
            to_: address;
            tickets_to_import: Array<ticket>;
        }>) => Promise<void>;
    lambda_export: (
        ticketsToExport: Array<{
            from_: address;
            token_id: nat;
            amount: nat;
        }>,
        destination: Instruction[],
    ) => Promise<void>;
    export_ticket: (
        destination: {Some: address} | null,
        requests: Array<{
            to_: address;
            ticketsToExport: Array<{
                from_: address;
                token_id: nat;
                amount: nat;
            }>;
        }>,
    ) => Promise<void>;
    approve: (param: Array<{
            owner: address;
            spender: address;
            token_id: nat;
            old_value: nat;
            new_value: nat;
        }>) => Promise<void>;
    addOperator: (
        owner: address,
        operator: address,
        token_id: nat,
    ) => Promise<void>;
    removeOperator: (
        owner: address,
        operator: address,
        token_id: nat,
    ) => Promise<void>;
    balance_of: (
        requests: Array<{
            owner: address;
            token_id: nat;
        }>,
        callback: contract,
    ) => Promise<void>;
    transfer: (param: Array<{
            from_: address;
            txs: Array<{
                to_: address;
                amount: nat;
                token_id: nat;
            }>;
        }>) => Promise<void>;
    mint_fn: (
        to_: address,
        token_id: nat,
        amount: nat,
        token_info: {Some: MMap<string, bytes>} | null,
    ) => Promise<void>;
};

type MethodsObject = {
    import_ticket: (param: Array<{
            to_: address;
            tickets_to_import: Array<ticket>;
        }>) => Promise<void>;
    lambda_export: (params: {
        ticketsToExport: Array<{
            from_: address;
            token_id: nat;
            amount: nat;
        }>,
        destination: Instruction[],
    }) => Promise<void>;
    export_ticket: (params: {
        destination: {Some: address} | null,
        requests: Array<{
            to_: address;
            ticketsToExport: Array<{
                from_: address;
                token_id: nat;
                amount: nat;
            }>;
        }>,
    }) => Promise<void>;
    approve: (param: Array<{
            owner: address;
            spender: address;
            token_id: nat;
            old_value: nat;
            new_value: nat;
        }>) => Promise<void>;
    addOperator: (params: {
        owner: address,
        operator: address,
        token_id: nat,
    }) => Promise<void>;
    removeOperator: (params: {
        owner: address,
        operator: address,
        token_id: nat,
    }) => Promise<void>;
    balance_of: (params: {
        requests: Array<{
            owner: address;
            token_id: nat;
        }>,
        callback: contract,
    }) => Promise<void>;
    transfer: (param: Array<{
            from_: address;
            txs: Array<{
                to_: address;
                amount: nat;
                token_id: nat;
            }>;
        }>) => Promise<void>;
    mint_fn: (params: {
        to_: address,
        token_id: nat,
        amount: nat,
        token_info: {Some: MMap<string, bytes>} | null,
    }) => Promise<void>;
};

type contractTypes = { methods: Methods, methodsObject: MethodsObject, storage: Storage, code: { __type: 'NFTCode', protocol: string, code: object[] } };
export type NFTContractType = ContractAbstractionFromContractType<contractTypes>;
export type NFTWalletType = WalletContractAbstractionFromContractType<contractTypes>;
