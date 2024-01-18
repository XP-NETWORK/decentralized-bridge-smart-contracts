# The NFT Bridge Main Smart Contract

The smart contracts are built using [`anchor_lang` framework](https://docs.rs/anchor-lang/latest/anchor_lang/). It implements the programming patterns customary to 
- [Solidity](https://docs.soliditylang.org/en/v0.8.10/#), 
- [Truffle](https://www.trufflesuite.com/), 
- [web3](https://github.com/ethereum/web3.js),
- [!Ink](https://github.com/paritytech/ink)

## Imports

Local Modules:
```rust
/// The module with the Bridge related
/// Structs:
///     Bridge              - The bridge internal fields (variables)
///     Transaction         - A TX internal variables
///     TransactionAccount  - A TX Account internal variables
/// Implementations:
///     From<&Transaction> for anchor_lang::solana_program::instruction::Instruction
///     From<&TransactionAccount> for AccountMeta (imported with the prelude crate)
///     From<&AccountMeta> for TransactionAccount
pub mod state; // For details see the corresponding document.

/// Contains an enum with the 
/// Error types & 
/// corresponding error messages
pub mod error;
```

The [prelude](https://docs.rs/anchor-lang/latest/anchor_lang/prelude/index.html) contains all commonly used components of the crate. It is recomended to import it like so: 
```rust
anchor_lang::prelude::*;.
```

A reexported library [solana_program](https://docs.rs/solana-program/1.9.4/solana_program/index.html) is responsible for the hashing, encryption & system related functionality:

```rust
use anchor_lang::solana_program;
```

```rust
/// https://docs.rs/solana-program/1.9.4/solana_program/system_program/index.html
/// Stores the static program ID
/// Functions:
///     check_id	Confirms that a given pubkey is equivalent to the program ID
///     id	        Returns the program ID
use anchor_lang::solana_program::system_program;

/// A directive for a single invocation of a Solana program.
use anchor_lang::solana_program::instruction::Instruction;

/// A C representation of Rust’s std::option::Option
use anchor_lang::solana_program::program_option::COption;

/// https://docs.rs/anchor-spl/0.5.0/anchor_spl/token/index.html
/// Token related functionality
use anchor_spl::token::{self, Mint, MintTo, Burn, Transfer, TokenAccount};

/// https://doc.rust-lang.org/std/convert/trait.Into.html
/// A value-to-value conversion that consumes the input value. The opposite of From.
use std::convert::Into;

/// The local state module structs
use state::{Bridge, Transaction, TransactionAccount};

/// The local error enum with the Error types & their messages
use error::ErrorCode;
```

## `declare_id!`
A convenience macro to declare a static public key and functions to interact with it<br>
[Input](https://docs.rs/solana-program/1.9.4/solana_program/macro.declare_id.html): a single literal base58 string representation of a program’s id

## `#[program]`
This attribute from `anchor_lang` defines the module containing all instruction handlers defining all entries into a Solana program.

## pub mod xp_bridge {...} LL20-291
The main bridge module.

```rust
/// xp_bridge import of
use anchor_lang::solana_program::{
    program::invoke,    // Invoke a cross-program instruction.
    system_instruction  // https://docs.rs/solana-program/1.9.4/solana_program/system_instruction/index.html
};
```

## `use super::*;`
Import the `anchor_lang` framework's components.

