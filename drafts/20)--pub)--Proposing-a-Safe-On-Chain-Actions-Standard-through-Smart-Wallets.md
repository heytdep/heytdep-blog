`19/08/2024`

In a [recent post](https://blog.xycloo.com/blog/blend-bot-with-zephyr-and-smart-accounts), XyclooLabs showed a proof-of-concept for using smart accounts for on-chain actions deployed on the cloud, and not only.

If you want to learn more about smart accounts on stellar and why they should be used for on-chain actions and automation I
recommend reading the paragraphs *"What are smart accounts on Stellar?"* and *"Why use smart accounts for cloud on-chain actions?"*
from XyclooLabs' post. 

The TLDR; is that since smart accounts enable to specify various authorization policies given different authorities (eg:
signers), then we should rely on such policies for:

1. **Cloud on-chain actions**: Deploying on chain actions on the cloud can be very advantageous in terms of dev experience and efficiency, 
but you need to trust the cloud operator with your secret key. In such situations using a smart account with a special authorization
policy for a signer that you're trusting the cloud operator with. This allows you to safely administrate your funds even though
they can be accessed by a cloud bot following a certain set of rules. 

2. **Self-hosted on-chain actions**: While in self-hosted on-chain actions you don't have to trust a 3rd party with any secret key,
reducing the amount of times your account's master secret key is used and accessed is never a bad practice, especially with on-chain
real-time actions where the secret key/secret key decoder has to be hardcoded in plain text in the program. 

Additionally, smart accounts have the advantage of being smart contracts themselves allowing to execute actions atomically
without relying on additional contracts as we will see in this standard proposal draft.

# Motivation

The motivation behing this draft is to specify a common behaviour and authorization structure among smart wallets that are used
for on-chain actions. More specifically, this proposal includes:

1. A rust derive macro tthat can be plugged in in any smart account to make it compatible to the standard.

2. A generic implementation of a smart account that enforces this standard along with the above macro.

3. A client-side implementation of a bot interacting with the smart account.

# Specification

The specification puts compatibility with any existing smart wallet at the center and is split in two parts:

1. authorizing and verifying allowed actions.

2. multiop function for executing multiple actions atomically.

## Authorizing actions.

### The standard actions data type.

The standard proposes a type whose name can be macro-secified (defaults to `StandardAllowedActions`) which wraps a series
of allowed contracts identified by an assignable numeric id and a set of allowed actions whose name can also be specified in
the macro (defaults to `StandardAction`):

```rust
// Note: Name defaults to StandardAllowedActions if not specified
#[contracttype]
pub struct #generic_typename_attr {
    contracts: Map<u32, Address>,
    actions: Map<Symbol, StandardAction>,
    deploy: bool,
}

// Note: Name defaults to StandardAction if not specified
#[contracttype]
pub struct #specific_typename_attr {
    allowed_contracts: Vec<u32>,
    allowed_args: Map<u32, Vec<Val>>,
}
```

The rationale behind the `StandardAllowedActions` structure is the following:

1. deploy can be either turned on or off, but it might be good to look into further deploy customization if needed.

2. actions are identified by the invoked function name, and a single function name can be allowed for multiple contract addresses,
specified with the contract's id in the `contracts` map.

3. actions can also be authorized depending on the invocation's argument. Allowed arguments at certain indexes are whitelisted
within the `allowed_args` field, where the key is the argument index and the value is an array of authorized argument values.

Below is the implementation that verifies if the configuration allows for a certain context:

```rust
impl StandardAllowedActions {
    pub fn is_allowed(&self, env: &Env, context: Context) -> bool {
        match context {
            Context::CreateContractHostFn(_) => self.deploy,
            Context::Contract(contract_ctx) => {
                let args = contract_ctx.args;
                let contract = contract_ctx.contract;
                let fname = contract_ctx.fn_name;

                let mut is_allowed = false;

                if let Some(allowed) = self.actions.get(fname) {
                    for contract_idx in allowed.allowed_contracts {
                        if let Some(address) = self.contracts.get(contract_idx) {
                            if contract == address {
                                let mut current_args_approved = true;
                                for (idx, allowed_args) in allowed.allowed_args.clone().into_iter() {
                                    if let Some(current_arg) = args.get(idx) {
                                        if allowed_args.iter().find(|x| vec![&env, x.clone()] == vec![&env, current_arg.clone()]).is_none() {
                                            current_args_approved = false
                                        }
                                    } else {
                                        current_args_approved = false
                                    }
                                }

                                if current_args_approved {
                                    is_allowed = true
                                }
                            }
                        }
                    }
                }

                is_allowed
            }
        }
    }
}

pub fn is_standard_allowed_action(env: &Env, ctxs: Vec<Context>) -> bool {
    let actions: StandardAllowedActions = env.storage().instance().get(&symbol_short!(#actions_storage_attr)).unwrap();
    let mut allowed = true;

    for ctx in ctxs {
        if !actions.is_allowed(env, ctx) {
            allowed = false
        }
    }

    allowed
}
```

### Setting the allowed actions

The proposed macro also implements a contract function to set the allowed actions:

```rust
#[contractimpl]
impl #struct_name {
    /// Set the contract's allowed on-chain actions.
    pub fn #actions_fname_attr(env: Env, actions: StandardAllowedActions) {
        env.current_contract_address().require_auth();
        env.storage().instance().set(&symbol_short!(#actions_storage_attr), &actions);
    }
// ...
}
```

## Multi-op Invocation

As mentioned in the beginning, an advantage of using smart accounts for on-chain automated actions is leveraging
it to perform multiple operations atomically. For example, say that you need to perform an automated arb trading action:

1. Buy token A from dex 1.

2. Sell token A in dex 2.

In order to perform the swap atomically you'd need a separate smart contract that bundles the two operations. Instead,
the proposed macro adds a new function (defaults to `saa_invoke` as in smart account action invoke) that bundles arbitrary
contract calls in a single transaction:

```rust
#[contractimpl]
impl #struct_name {
    pub fn #invoke_fname_attr(env: Env, actions: Vec<(Address, Symbol, Vec<Val>)>) {
        env.current_contract_address().require_auth();
        for action in actions {
            let contract = action.0;
            let fname = action.1;
            let args = action.2;

            let _: Val = env.invoke_contract(&contract, &fname, args);
        }

        ()
    }
}
```

<hr/>

# Implementation on a Generic Smart Account

Implementing the macro is simple, but for the sake of compatibility there is some behaviour that needs to be defined by the
implementation:

1. **Authentication**: the implementer must implement the logic to add and identify the "special" signer.

2. **Macro attributes**: to counter a potential types/functions naming collisions with the existing implementation
the macro accepts a series of optional parameters to change the names:

```
attributes(
    actions_fname,
    invoke_fname,
    generic_typename,
    specific_typename,
    actions_storage,
)
```

## Example

```rust
#![no_std]

use extension_macro::StandardActions;
use soroban_sdk::{
    auth::{Context, CustomAccountInterface},
    contract, contracterror, contractimpl, contracttype,
    crypto::Hash,
    symbol_short, vec, Address, BytesN, Env, Map, Symbol, Val, Vec,
};

#[contract]
#[derive(StandardActions)]
struct AccountContract;

#[contracttype]
#[derive(Clone)]
pub struct Signature {
    pub public_key: BytesN<32>,
    pub signature: BytesN<64>,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Signer(BytesN<32>),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AccError {
    UnknownSigner = 1,
    InvalidContext = 2,
}

#[contractimpl]
impl AccountContract {
    // Add other init params here.
    pub fn init(env: Env, master_signer: BytesN<32>, standard_signer: BytesN<32>) {
        env.storage()
            .instance()
            .set(&DataKey::Signer(master_signer), &true);
        env.storage()
            .instance()
            .set(&DataKey::Signer(standard_signer), &false);
    }
}

#[contractimpl]
impl CustomAccountInterface for AccountContract {
    type Signature = Signature;
    type Error = AccError;

    #[allow(non_snake_case)]
    fn __check_auth(
        env: Env,
        signature_payload: Hash<32>,
        signature: Signature,
        auth_contexts: Vec<Context>,
    ) -> Result<(), AccError> {
        let is_master = authenticate(&env, &signature_payload, &signature)?;
        let mut result = Err(AccError::InvalidContext);

        if is_master {
            result = Ok(())
        } else {
            if is_standard_allowed_action(&env, auth_contexts) {
                result = Ok(())
            };
        }

        result
    }
}

fn authenticate(
    env: &Env,
    signature_payload: &Hash<32>,
    signature: &Signature,
) -> Result<bool, AccError> {
    let is_master = if let Some(is_master) = env
        .storage()
        .instance()
        .get(&DataKey::Signer(signature.public_key.clone()))
    {
        is_master
    } else {
        return Err(AccError::UnknownSigner);
    };

    env.crypto().ed25519_verify(
        &signature.public_key,
        &signature_payload.clone().into(),
        &signature.signature,
    );

    Ok(is_master)
}

mod test;
```

# Full proposed macro

```rust
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    self, parse_macro_input, Attribute, DeriveInput, Expr, ExprLit, Ident, Lit
};

fn get_attribute(attrs: Vec<Attribute>, attr_name: &str, default_to: &str) -> Ident {
    let ident_source = get_attribute_string(attrs, attr_name, default_to);
    Ident::new(&ident_source, Span::call_site())
}

fn get_attribute_string(attrs: Vec<Attribute>, attr_name: &str, default_to: &str) -> String {
    attrs
        .iter()
        .find_map(|attr| {
            if attr.path().is_ident(attr_name) {
                let value: Expr = attr.parse_args().unwrap();
                if let Expr::Lit(ExprLit { lit, .. }) = value {
                    if let Lit::Str(value) = lit {
                        return Some(value.value());
                    } else {
                        panic!("Invalid lit type")
                    }
                } else {
                    panic!("Invalid type")
                }
            } else {
                panic!("No provided, defaulting to standard")
            }
        })
        .unwrap_or(default_to.to_string())
}

#[proc_macro_derive(
    StandardActions,
    attributes(
        actions_fname,
        invoke_fname,
        generic_typename,
        specific_typename,
        actions_storage,
    )
)]
pub fn standard_actions_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let actions_fname_attr = get_attribute(
        input.attrs.clone(),
        "actions_fname",
        "set_standard_allowed_actions",
    );
    let invoke_fname_attr = get_attribute(input.attrs.clone(), "invoke_fname", "saa_invoke");
    let generic_typename_attr = get_attribute(
        input.attrs.clone(),
        "generic_typename",
        "StandardAllowedActions",
    );
    let specific_typename_attr =
        get_attribute(input.attrs.clone(), "specific_typename", "StandardAction");
    let actions_storage_attr = get_attribute_string(input.attrs.clone(), "actions_storage", "SAA");

    let expanded = quote! {
        #[contracttype]
        pub struct #generic_typename_attr {
            contracts: Map<u32, Address>,
            actions: Map<Symbol, StandardAction>,
            deploy: bool,
        }

        #[contracttype]
        pub struct #specific_typename_attr {
            allowed_contracts: Vec<u32>,
            allowed_args: Map<u32, Vec<Val>>,
        }

        impl StandardAllowedActions {
            pub fn is_allowed(&self, env: &Env, context: Context) -> bool {
                match context {
                    Context::CreateContractHostFn(_) => self.deploy,
                    Context::Contract(contract_ctx) => {
                        let args = contract_ctx.args;
                        let contract = contract_ctx.contract;
                        let fname = contract_ctx.fn_name;

                        let mut is_allowed = false;

                        if let Some(allowed) = self.actions.get(fname) {
                            for contract_idx in allowed.allowed_contracts {
                                if let Some(address) = self.contracts.get(contract_idx) {
                                    if contract == address {
                                        let mut current_args_approved = true;
                                        for (idx, allowed_args) in allowed.allowed_args.clone().into_iter() {
                                            if let Some(current_arg) = args.get(idx) {
                                                if allowed_args.iter().find(|x| vec![&env, x.clone()] == vec![&env, current_arg.clone()]).is_none() {
                                                    current_args_approved = false
                                                }
                                            } else {
                                                current_args_approved = false
                                            }
                                        }

                                        if current_args_approved {
                                            is_allowed = true
                                        }
                                    }
                                }
                            }
                        }

                        is_allowed
                    }
                }
            }
        }

        pub fn is_standard_allowed_action(env: &Env, ctxs: Vec<Context>) -> bool {
            let actions: StandardAllowedActions = env.storage().instance().get(&symbol_short!(#actions_storage_attr)).unwrap();
            let mut allowed = true;

            for ctx in ctxs {
                if !actions.is_allowed(env, ctx) {
                    allowed = false
                }
            }

            allowed
        }

        #[contractimpl]
        impl #struct_name {
            /// Set the contract's allowed on-chain actions.
            pub fn #actions_fname_attr(env: Env, actions: StandardAllowedActions) {
                env.current_contract_address().require_auth();
                env.storage().instance().set(&symbol_short!(#actions_storage_attr), &actions);
            }

            pub fn #invoke_fname_attr(env: Env, actions: Vec<(Address, Symbol, Vec<Val>)>) {
                env.current_contract_address().require_auth();
                for action in actions {
                    let contract = action.0;
                    let fname = action.1;
                    let args = action.2;

                    let _: Val = env.invoke_contract(&contract, &fname, args);
                }

                ()
            }
        }
    };

    TokenStream::from(expanded)
}
```

<hr/>

# Client side implementations

An advantage of having a standard for on-chain actions on smart accounts is the ease of implementation client-side. For example,
we've added an experimental function to a not-yet released version of the [Zephyr Rust SDK](https://docs.rs/zephyr-sdk/latest/zephyr_sdk/) that allows to seamlessly work with
this standard and perform on-chain actions:

1. example contract that implements the standard: `CCMJ32EN7E4AKK6K6MGKSVRZPER6RWU3RKDM4VS43T2KC365AMLW3MI2`.

2. `saa_invoke` transaction example: [https://stellar.expert/explorer/testnet/tx/eed981695467cfad107267a1a2de20d81a81266dde998ff58071bac07e0f922d](https://stellar.expert/explorer/testnet/tx/eed981695467cfad107267a1a2de20d81a81266dde998ff58071bac07e0f922d).

3. configuring the authorized actions transaction example: [https://stellar.expert/explorer/testnet/tx/a6e0ca3f466b42508ba12f71073b5a4f10e15c4318067f914e2677c738721090](https://stellar.expert/explorer/testnet/tx/a6e0ca3f466b42508ba12f71073b5a4f10e15c4318067f914e2677c738721090).

## Zephyr implementation

### Performing on-chain actions

This example is on-demand in this case, but can be easily used within an automated workflow as the one described
in [XyclooLabs's post](https://blog.xycloo.com/blog/blend-bot-with-zephyr-and-smart-accounts).

```rust

#[no_mangle]
pub extern "C" fn do_action() {
    let env = EnvClient::empty();
    let wallet_address = address_from_str(&env, SMART_ACCOUNT);
    let ybx_address = address_from_str(&env, &YBX_CONTRACT);
    
    let action: (Address, Symbol, soroban_sdk::Vec<Val>) = {
        let fname = "submit";
        let map: Map<Symbol, Val> = map![
            &env.soroban(),
            (
                Symbol::new(&env.soroban(), "request_type"),
                2_u32.into_val(env.soroban()),
            ),
            (
                Symbol::new(&env.soroban(), "address"),
                Address::from_string(&zephyr_sdk::soroban_sdk::String::from_str(
                    &env.soroban(),
                    "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
                ))
                .into_val(env.soroban()),
            ),
            (
                Symbol::new(&env.soroban(), "amount"),
                100_000_000_i128.into_val(env.soroban()),
            )
        ];
        
        let args: soroban_sdk::Vec<Val> = vec![
            &env.soroban(),
            wallet_address.into_val(env.soroban()),
            wallet_address.into_val(env.soroban()),
            wallet_address.into_val(env.soroban()),
            vec![&env.soroban(), map].into_val(env.soroban()),
        ];
        
        (ybx_address, Symbol::new(&env.soroban(), fname), args)
    };

    let saa_actions: soroban_sdk::Vec<(Address, Symbol, soroban_sdk::Vec<Val>)> = vec![&env.soroban(), action];
    //let bytes = env.to_scval(saa_actions.clone()).to_xdr_base64(Limits::none()).unwrap();
    //env.log().debug(format!("{:?}", bytes), None);

    env.log().debug("Executing smart account transaction", None);
    execute_smart_account_transaction(
        &env,
        &NETWORK,
        "https://horizon-testnet.stellar.org/transactions",
        &SOURCE_ACCOUNT,
        &SOUCRE_SECRET,
        &SMART_ACCOUNT,
        "saa_invoke",
        vec![&env.soroban(), saa_actions.into_val(env.soroban())],
        SIGNATURE_DURATION,
        &MERCURY_SECRET,
        &SMART_ACCOUNT,
        &SMART_ACCOUNT_HASH,
        false,
        INSTRUCTIONS_FIX,
        WRITE_BYTES_FIX,
        READ_BYTES_FIX,
        RESOURCE_FEE_FIX,
        FEE_FIX,
        build_sig
    );

    env.conclude("Successfully sent transaction")
}
```

### Setting allowed actions

```rust
#[no_mangle]
pub extern "C" fn set_actions() {
    let env = EnvClient::empty();
    
    let fname = "set_standard_allowed_actions";
    let wallet_address =
        Address::from_string(&SorobanString::from_str(&env.soroban(), &SMART_ACCOUNT));

    let mut allowed_contracts = Map::new(&env.soroban());
    allowed_contracts.set(
        1_u32,
        Address::from_string(&SorobanString::from_str(&env.soroban(), &YBX_CONTRACT)),
    );
    allowed_contracts.set(2_u32, wallet_address.clone());
    allowed_contracts.set(3_u32, address_from_str(&env, "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"));

    let mut allowed_actions = Map::new(env.soroban());
    add_allowed_action(
        &env,
        &mut allowed_actions,
        "submit",
        vec![&env.soroban(), 1_u32],
        Map::<u32, soroban_sdk::Vec<Val>>::from_array(
            &env.soroban(),
            [
                (
                    0_u32,
                    vec![
                        &env.soroban(),
                        wallet_address.clone().into_val(env.soroban()),
                    ],
                ),
                (
                    1_u32,
                    vec![
                        &env.soroban(),
                        wallet_address.clone().into_val(env.soroban()),
                    ],
                ),
                (
                    2_u32,
                    vec![
                        &env.soroban(),
                        wallet_address.clone().into_val(env.soroban()),
                    ],
                ),
            ],
        ),
    );

    add_allowed_action(
        &env,
        &mut allowed_actions,
        "saa_invoke",
        vec![&env.soroban(), 2_u32],
        Map::new(&env.soroban())
    );

    add_allowed_action(
        &env,
        &mut allowed_actions,
        "transfer",
        vec![&env.soroban(), 3_u32],
        Map::new(&env.soroban())
    );

    let standard_allowed_actions = build_standard_actions(&env, allowed_contracts, allowed_actions, false);
    
    let arguments = vec![
        &env.soroban(),
        standard_allowed_actions.into_val(env.soroban()),
    ];

    env.log().debug("Executing smart account transaction", None);
    execute_smart_account_transaction(
        &env,
        &NETWORK,
        "https://horizon-testnet.stellar.org/transactions",
        &SOURCE_ACCOUNT,
        &SOUCRE_SECRET,
        &SMART_ACCOUNT,
        &fname,
        arguments,
        SIGNATURE_DURATION,
        &SOUCRE_SECRET,
        &SMART_ACCOUNT,
        &SMART_ACCOUNT_HASH,
        false,
        INSTRUCTIONS_FIX,
        WRITE_BYTES_FIX,
        READ_BYTES_FIX,
        RESOURCE_FEE_FIX,
        FEE_FIX,
        build_sig
    );

    env.conclude("Successfully sent transaction")
}
```
