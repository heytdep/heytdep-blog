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
