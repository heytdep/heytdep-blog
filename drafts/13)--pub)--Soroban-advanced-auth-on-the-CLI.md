# Building and using signatures in Futurenet/Local Networks for the advanced auth + Script for building signatures

> Before you read this, I recommend reading the [previous submission](https://github.com/stellar/sorobanathon/discussions/38) if you don't yet know the basics of using the CLI and Futurenet/Local Network. 

# Introduction
The advanced auth mechanism is a key compoment for smart contracts that rely on the user's authorization to do something. For example, when calling the `xfer` fn, the token contract relies on the fact that the user transferring money out of their balance has given the authorization to the contract to do so. In our [previous sorobanathon submission](https://github.com/stellar/sorobanathon/discussions/38) we wrote about using `Signature::Invoker`, but there are also situations where the user doesn't invoke the contract direclty. For example, if we where to invoke the `xfer` fn from a contract (contract A), we would need the user to provide a signature which contract A would then pass to the token contract. In this post, we'll talk about building signatures and how to pass them to a contract. I (@heytdep) have also written [a tool to build signatures](https://github.com/Xycloo/soroban-build-signatures/blob/main/src/main.rs) given a stellar keypair, action symbol, contract, and arguments. We will use this script to build the signatures for this post. 

# Understanding ed25519 signatures
Ed25519 signatures are the product of signing a message (Payload) with an ed25519 keypair. Since signatures are exposed to external observers, the payload they contain must be very specific to the action they are allowing a contract to perform in order to not allow attackers to re-use these signatures. For example, if we provide a signature for the `xfer` fn, the payload contains 3 "sections":
- the id of contract that is allowed to perform the `xfer` on behalf of the user.
- the name of the action that is performed (`symbol!("xfer")`).
- custom arguments, like who is going to receive the token, how much, and the nonce of the user (to prevent re-playing the same signature).

We also have to rember that we need the environment in which we are building the signature to have a network passphrase matching the passphrase of the network we want to use this signature in.

All of this needs to be signed by a keypair, performed by the `soroban_auth::testutils::ed25519::sign` function:

```rust
let sig = soroban_auth::testutils::ed25519::sign(&env, &kp, &contract_id, action, args);
```

# Building keypairs
There currently isn't a way of creating ed25519 keypairs in the auth sdk, so we'll have to create it on our own. Fortunately, you can use the script I mentioned before:

```rust
fn main() {
    let env = build_env("Standalone Network ; February 2017".to_string());

    let public_encoded = "GA63NQJB6SXHDVOI3NXP4GM3K5MB4KLTX6R4YK2KKXY4DM27ZNUOVJYY".to_string(); // this a test keypair that doesn't hold any value, you should never use this specific keypair since it may put your account's balance at risk.
    let secret_encoded = "SC2ZVG244UNKKBEKAQLEFAS2AU4XGEX5TXCXBTJZ6DXVU5MJ4E4FRKF4".to_string();

    let (kp_id, kp) = ed25519_utils::build_kp(
        &env,
        &decode_pub(public_encoded),
        &decode_secret(secret_encoded),
    );
	
	...
```

As you can see, we build the keypair (`kp`) with the `build_kp` fn, which accepts as arguments the strdecoded value of both the public and secret stellar keys.


# Building the Signature
Once we have built our keypair, we are ready to sign a payload:

```rust
fn main() {
    let env = build_env("Standalone Network ; February 2017".to_string());

    let public_encoded = "GA63NQJB6SXHDVOI3NXP4GM3K5MB4KLTX6R4YK2KKXY4DM27ZNUOVJYY".to_string();
    let secret_encoded = "SC2ZVG244UNKKBEKAQLEFAS2AU4XGEX5TXCXBTJZ6DXVU5MJ4E4FRKF4".to_string();

    let (kp_id, kp) = ed25519_utils::build_kp(
        &env,
        &decode_pub(public_encoded),
        &decode_secret(secret_encoded),
    );

    let contract_id =
        bytesn!(&env, 0x69f7e580340b3f963e56a40a11a4bc89264b53583fc92e65ef44efd051ab5a9b); // contract that can use this signature
    let action = symbol!("change"); // action
    let args = (); // args, since there is not measure to prevent re-playing this signature is insecure

    let sig = soroban_auth::testutils::ed25519::sign(&env, &kp, &contract_id, action, args);
    std::println!("{:?}", sig);
}

```

This will output the following signature:

```
Ed25519(Ed25519Signature { public_key: BytesN<32>(61, 182, 193, 33, 244, 174, 113, 213, 200, 219, 110, 254, 25, 155, 87, 88, 30, 41, 115, 191, 163, 204, 43, 74, 85, 241, 193, 179, 95, 203, 104, 234), signature: BytesN<64>(18, 121, 165, 102, 188, 133, 109, 43, 68, 70, 121, 63, 136, 175, 143, 101, 140, 147, 75, 248, 123, 132, 174, 49, 89, 56, 147, 230, 180, 30, 90, 45, 1, 206, 179, 48, 195, 170, 86, 67, 224, 39, 151, 217, 84, 253, 99, 111, 171, 219, 162, 151, 201, 150, 191, 130, 85, 140, 3, 240, 95, 171, 170, 7) })
```

We can now use this signature in a contract that needs to verify `verify(&e, &sig, symbol!("change"), ());`. So a contract that needs to verify that the user has a signature for the `"change"` action. Again, this is an insecure signature, to make it secure you simply need to add a couple of params when using the `verify` fn, for example, to make it secure we could use something like `verify(&e, &sig, symbol!("change"), (key, value, nonce));`, where the nonce is relative to a user and is stored in the contract's data.

# Writing the contract
Let's write a very simple contract that verifies a certain signature (note that we have a `Signature` argument:

```rust
#![no_std]
use soroban_auth::{verify, Ed25519Signature, Identifier, Signature};
use soroban_sdk::{contractimpl, contracttype, symbol, BigInt, Bytes, Env};

pub struct ExampleContract;

#[contracttype]
pub enum DataKey {
    Nonce(Identifier),
}

#[contractimpl]
impl ExampleContract {
	pub fn test_sig(e: Env, sig: Signature, key: Bytes, val: Bytes) {
        let nonce = get_nonce(&e, sig.identifier(&e));
        verify(&e, &sig, symbol!("change"), (key, val, nonce.clone()));
        e.data().set(DataKey::Nonce(sig.identifier(&e)), nonce + 1)
    }

    pub fn get(e: Env, key: Bytes) -> Identifier {
        e.data()
            .get(key)
            .unwrap_or_else(|| panic!("Key does not exist"))
            .unwrap()
    }

    pub fn nonce(e: Env, id: Identifier) -> BigInt {
        get_nonce(&e, id)
    }
}

fn get_nonce(e: &Env, id: Identifier) -> BigInt {
    e.data()
        .get(DataKey::Nonce(id))
        .unwrap_or_else(|| Ok(BigInt::zero(e)))
        .unwrap()
}

#[cfg(test)]
mod test;

```

A successful test looks like this:

```rust
use crate::{ExampleContract, ExampleContractClient};
use soroban_auth::Signature;
use soroban_sdk::{
    bytes, bytesn, symbol,
    testutils::{Ledger, LedgerInfo},
    BigInt, Env,
};

extern crate std;

#[test]
fn test_use_advanced_auth() {
    let e = Env::default();

    e.ledger().set(LedgerInfo {
        timestamp: 1668106305,
        protocol_version: 20,
        sequence_number: 10,
        network_passphrase: std::vec![
            83, 116, 97, 110, 100, 97, 108, 111, 110, 101, 32, 78, 101, 116, 119, 111, 114, 107,
            32, 59, 32, 70, 101, 98, 114, 117, 97, 114, 121, 32, 50, 48, 49, 55,
        ],
        base_reserve: 10,
    });

    let (user_1_id, user_1_sign) = soroban_auth::testutils::ed25519::generate(&e);

    let contract_id = e.register_contract(
        &std::option::Option::Some(
            bytesn!(&e, 0x69f7e580340b3f963e56a40a11a4bc89264b53583fc92e65ef44efd051ab5a9b),
        ),
        ExampleContract,
    );
    let client = ExampleContractClient::new(&e, &contract_id);

    let nonce = BigInt::from_u32(&e, 0);
    let sig = soroban_auth::testutils::ed25519::sign(
        &e,
        &user_1_sign,
        &contract_id,
        symbol!("change"),
        (bytes!(&e, 0x7), bytes!(&e, 0x7), nonce),
    );

    //    std::println!("{:?}", sig);

    client.test_sig(&sig, &bytes!(&e, 0x7), &bytes!(&e, 0x7));
}

```

# Replicating the behaviour on Futurenet/Local Network
First, we build the wasm binary of the contract:
```bash
cargo +nightly build \  
    --target wasm32-unknown-unknown \
    --release \
    -Z build-std=std,panic_abort \
    -Z build-std-features=panic_immediate_abort
    Finished release [optimized] target(s) in 0.05s

```

Then we deploy it:

```bash
soroban deploy \
    --wasm target/wasm32-unknown-unknown/release/test_soroban_cli_futurenet.wasm --secret-key DEPLOYER_SECRET --rpc-url http://HOST_INSTANCE:8000/soroban/rpc --network-passphrase 'Standalone Network ; February 2017'


out:
success
9c17051a8d43f2e1e062e69980df8b41ebbf55a50065daee59c8b7a4720b10f8

```

Now that we have the contract ID we can invoke it. But before we need to build a valid signature:

```rust

fn main() {
    let env = build_env("Standalone Network ; February 2017".to_string());

    let public_encoded = "GA63NQJB6SXHDVOI3NXP4GM3K5MB4KLTX6R4YK2KKXY4DM27ZNUOVJYY".to_string();
    let secret_encoded = "SC2ZVG244UNKKBEKAQLEFAS2AU4XGEX5TXCXBTJZ6DXVU5MJ4E4FRKF4".to_string();

    let (kp_id, kp) = ed25519_utils::build_kp(
        &env,
        &decode_pub(public_encoded),
        &decode_secret(secret_encoded),
    );

    let contract_id =
        bytesn!(&env, 0x9c17051a8d43f2e1e062e69980df8b41ebbf55a50065daee59c8b7a4720b10f8);
    let action = symbol!("change");
    let args = (
        bytes!(&env, 0x68656c6c6f), // hex for "Hello"
        bytes!(&env, 0x68656c6c6f),
        BigInt::zero(&env),
    );

    let sig = soroban_auth::testutils::ed25519::sign(&env, &kp, &contract_id, action, args);
    std::println!("{:?}", sig);
}
```

We now have:

```
Ed25519(Ed25519Signature { public_key: BytesN<32>(61, 182, 193, 33, 244, 174, 113, 213, 200, 219, 110, 254, 25, 155, 87, 88, 30, 41, 115, 191, 163, 204, 43, 74, 85, 241, 193, 179, 95, 203, 104, 234), signature: BytesN<64>(80, 243, 125, 109, 171, 229, 144, 33, 237, 137, 67, 51, 145, 226, 27, 220, 108, 115, 33, 105, 164, 252, 164, 10, 138, 189, 216, 158, 36, 184, 129, 188, 86, 138, 46, 219, 103, 34, 185, 42, 97, 233, 238, 184, 231, 115, 182, 193, 211, 8, 164, 222, 196, 27, 30, 255, 200, 236, 179, 211, 250, 107, 123, 10) })
```

We have to convert both the public_key and signature bytes to a hex string, we can do it with [this go script](https://go.dev/play/p/uxOTa6cuPeI).

Then according to the structure of the `Ed25519Signature` (`Ed25519Signature { public_key: Bytes, signature: Bytes }`) struct, we build the JSON that we can pass as the signature argument:

```json
{
  "object": {
    "vec": [
      {
        "symbol": "Ed25519"
      },
      {
        "object": {
          "map": [
            {
              "key": {
                "symbol": "public_key"
              },
              "val": {
                "object": {
                  "bytes": "3db6c121f4ae71d5c8db6efe199b57581e2973bfa3cc2b4a55f1c1b35fcb68ea"
                }
              }
            },
            {
              "key": {
                "symbol": "signature"
              },
              "val": {
                "object": {
                  "bytes": "50f37d6dabe59021ed89433391e21bdc6c732169a4fca40a8abdd89e24b881bc568a2edb6722b92a61e9eeb8e773b6c1d308a4dec41b1effc8ecb3d3fa6b7b0a"
                }
              }
            }
          ]
        }
      }
    ]
  }
}
```

Finally, we invoke the contract:

```bash
soroban invoke \
  --id 9c17051a8d43f2e1e062e69980df8b41ebbf55a50065daee59c8b7a4720b10f8 \
  --secret-key SECRET \
  --rpc-url http://INSTANCE_HOST:8000/soroban/rpc \
  --network-passphrase 'Standalone Network ; February 2017' \
  --fn test_sig --arg '{"object":{"vec":[{"symbol":"Ed25519"},{"object":{"map":[{"key":{"symbol":"public_key"},"val":{"object":{"bytes":"3db6c121f4ae71d5c8db6efe199b57581e2973bfa3cc2b4a55f1c1b35fcb68ea"}}},{"key":{"symbol":"signature"},"val":{"object":{"bytes":"50f37d6dabe59021ed89433391e21bdc6c732169a4fca40a8abdd89e24b881bc568a2edb6722b92a61e9eeb8e773b6c1d308a4dec41b1effc8ecb3d3fa6b7b0a"}}}]}}]}}' --arg "68656c6c6f" --arg "68656c6c6f"

output:
success
null

```

As you can see, we just successfully verified an "Advanced Auth" signature!
