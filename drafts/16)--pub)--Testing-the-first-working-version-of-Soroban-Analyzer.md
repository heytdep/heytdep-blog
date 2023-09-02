`Jan 16, 2023`

Hey there! It's been a couple of months since I wrote here. But I'm back with an interesting hands-on article about using [Soroban Analyzer](https://github.com/xycloo/soroban-analyzer). 

# What is Soroban Analyzer
Soroban Analyzer is a CLI tool and a crate which you can use to detect gas inefficiencies in Soroban rust code.

The tool makes use of a slightly tweaked fork of Mozilla's [rust-code-analysis](https://github.com/xycloo/soroban-analyzer) crate to parse into nodes the Rust code.

The tool's workflow as I write the article can be described as follows:
1. Detect functions that access contract storage, loops, code blocks and add them to the `Storage` objsct.
2. Check if loops access contract storage indirectly (through what the tool found on step 1).
3. Check if a code block uses storage functions multiple times.

Later in the article I'll go deeper over why the tool does the above checks.

# Trying it out
Let's try out the tool by having it perform checks on a very simple contract that just stores a value (the supply) on the contract's data, which is changed by a function `supp(e: Env) -> i128` that returns the new supply.

We could write the contract as follows:

```rust
#![no_std]
use soroban_sdk::{contractimpl, contracttype, Env};

pub struct TestContract;

#[contracttype]
pub enum DataKey {
    Supply,
}

fn set_supply(e: &Env, s: i128) {
    e.storage().set(DataKey::Supply, s);
}

fn get_supply(e: &Env) -> i128 {
    e.storage().get(DataKey::Supply).unwrap().unwrap()
}

#[contractimpl]
impl TestContract {
    pub fn init(env: Env) {
        set_supply(&env, 10);
    }

    pub fn supp(e: Env) -> i128 {
        let supply = get_supply(&e);

        // supply decreases
        let amount = 2;
        set_supply(&e, supply - amount);

        get_supply(&e)
    }
}

mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test() {
        let env = Env::default();
        let contract_id = env.register_contract(None, TestContract);
        let client = TestContractClient::new(&env, &contract_id);

        env.budget().reset();

        client.init();

        let _new_supp = client.supp();

        extern crate std;
        std::println!("non optimized: \n {}", env.budget());
    }
}
```

Now it's time to try out soroban analyzer, and see if it's able to pick up something:

```
$ soroban-analyzer --all


[+] Soroban Analyzer started. Disclaimer: still under development, the tool's scope is currently very limited, expect bugs and breaking changes may occur. 
Report bugs or suggestions in the github issues page: https://github.com/xycloo/soroban-analyzer/issues.


 [DEBUG] Functions found directly or indirectly accessing contract state: 

> set_supply at line 11
> get_supply at line 15
> init at line 21
> supp at line 25
> test at line 41



[+] Starting checks 

[lib.rs] Lines 25-33: the function `get_supply` defined at line 15 accessed contract state and is used multiple times inside the block. It may be better to use `get_supply` once and save it in memory. 


```

The analyzer picked something up, let's see what it's about.

# Evaluating the results
The analzyer complains about the block 25-33, i.e the contract's `supp` function, and it tells us that `the function 'get_supply' defined at line 15 accessed contract state and is used multiple times inside the block.` Later on, the analyzer suggests to `use 'get_supply' once and save it in memory`.

The analyzer tells us this since the `get_supply` function accesses contract storage (`e.storage().get(DataKey::Supply)`), and accessing contract storage is more expensive than other in-memory operations.

In this case, the analyzer is saying that using `get_supply` another time at line 32 is unnecessary and expensive, and suggests us to simply re-use the `supply` var we defined at line 26. So something like this:

```rust
pub fn supp(e: Env) -> i128 {
	let supply = get_supply(&e);

    // supply decreases
    let amount = 2;
    set_supply(&e, supply - amount);

    supply - amount
}
```

As you can see, rather than re-accessing the storage to return the new supply, we simply determine it with a subtraction operation. Which results cheaper:

```bash
optimized:
Costs:
- CPU Instructions: 42737
- Memory Bytes: 4547

non optimized:
Costs:
- CPU Instructions: 52062
- Memory Bytes: 5573
```

<details><summary>See test code</summary>

```rust
#![no_std]
use soroban_sdk::{contractimpl, contracttype, Env};

pub struct TestContract;

#[contracttype]
pub enum DataKey {
    Supply,
}

fn set_supply(e: &Env, s: i128) {
    e.storage().set(DataKey::Supply, s);
}

fn get_supply(e: &Env) -> i128 {
    e.storage().get(DataKey::Supply).unwrap().unwrap()
}

#[contractimpl]
impl TestContract {
    pub fn init(env: Env) {
        set_supply(&env, 10);
    }

    pub fn supp(e: Env) -> i128 {
        let supply = get_supply(&e);

        // supply decreases
        let amount = 2;
        set_supply(&e, supply - amount);

        get_supply(&e)
    }

    pub fn supp_opt(e: Env) -> i128 {
        let supply = get_supply(&e);

        // supply decreases
        let amount = 2;
        set_supply(&e, supply - amount);

        supply - amount
    }
}

mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test() {
        let env = Env::default();
        let contract_id = env.register_contract(None, TestContract);
        let client = TestContractClient::new(&env, &contract_id);

        env.budget().reset();

        client.init();

        let _new_supp = client.supp();

        extern crate std;
        std::println!("non optimized: \n {}", env.budget());
    }

    #[test]
    fn test_opt() {
        let env = Env::default();
        let contract_id = env.register_contract(None, TestContract);
        let client = TestContractClient::new(&env, &contract_id);

        env.budget().reset();

        client.init();

        let _new_supp = client.supp_opt();

        extern crate std;
        std::println!("optimized: \n {}", env.budget());
    }
}
```

</details>

# Conclusion

There's still a lot of work to do on this tool, and if you have any suggestions, please let us know by opening an issue [here](https://github.com/xycloo/soroban-analyzer/issues).
