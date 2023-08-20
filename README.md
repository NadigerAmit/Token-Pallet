# Token Pallet

This Substrate-based pallet provides functionality for managing and interacting with tokens on the blockchain. It implements token creation, ownership, transfers, burning, and buying.

## Features

- Create unique tokens with a unique identifier.
- Manage ownership of tokens.
- Transfer tokens between accounts.
- Set and update token prices.
- Burn tokens to remove them from circulation.
- Buy tokens from other users based on the set price.

## Usage

1. **Create a Token:**
   Call the `create_token` function to create a new unique token.

2. **Burn a Token:**
   Call the `burn_token` function to burn (remove from circulation) an existing token.

3. **Transfer a Token:**
   Call the `transfer` function to transfer a token from one account to another.

4. **Set Token Price:**
   Call the `set_price` function to set or update the price of a token.

5. **Buy a Token:**
   Call the `buy_token` function to buy a token from another account based on the set price.

## Installation

1. Add the pallet as a dependency in your runtime's `Cargo.toml`:

   ```toml
   [dependencies]
   my_token_pallet = { path = "path/to/your/pallet" }

  ## Integrate the pallet into your runtime by adding it to the runtime's runtime/src/lib.rs:

1. Add the pallet as a dependency in your runtime's `Cargo.toml`:

   pub use my_token_pallet;
2. Configure the pallet in your runtime by implementing the Config trait:

License
This pallet is licensed under the Apache License, Version 2.0. See the LICENSE file for details.
