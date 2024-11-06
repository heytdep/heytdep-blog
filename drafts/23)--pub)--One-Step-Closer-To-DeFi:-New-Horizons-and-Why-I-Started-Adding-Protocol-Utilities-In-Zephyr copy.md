`05/11/2024`

Soroban needs to grow, let's accelerate that.

# Preamble

I think that anyone who is in the Soroban ecosystem right now notices that we're undergoing a stagnant situation. The tech is good,
consensus is strong, new full validators keep their uptimes, Meridian was great and we have applications solving problems for real users.

But no one seems to care. TVL is low, DeFi (in general) is ATL, there's little soroban volume, and there's almost no fuss about stellar anywhere
on mainstream crypto related news and discussions.

The reality is that **there is** work that is being done that we cannot see through on-chain activity just yet. Personally, part of this
work has been re-aligning my focus to work on problems that need to be solved today, not features that might be needed tomorrow.

In the upcoming months, you can likely expect more from me in terms of presence outside of the stellar
community (e.g CT) and also in terms of network growth. I've decided to better organize my timeline to (of course) 
keep prioritizing the improvement of [XyclooLabs](https://xycloo.clom/) products, but **also** to use our resources
to try driving more activity in the network through consumer-facing APIs and products that leverage the incredibly powerful infrastructure
that I've built over the last year.

What most don't realize is just how good it is to write Soroban smart contracts. I've starting digging in other virtual machines,
SDKs, APIs, clients, and I am yet to find an overall chain that feels slightly as good as stellar to build with.

But it's not the tech that matters, it's how it's being used and how it's presented to end-users and consumers. This is also why I've 
started working on adding protocol-specific interactions to Zephyr. This has the goal of aiding the creation of workers interacting
with the nework. And as token of how powerful Zephyr is in this situation, I present to you the **first open-source custodial
Blend positions (re)balancer**!

# Blend Rebalancer

This is a simple but really handy tool for those who like to earn safely in overcollateralized lending protocols. Ensuring that
your position remains at a certain health factor is a good strategy not to get liquidated and not having to pay attention
to your position every time relfector prices sync. 

The rebalancer allows you to choose a certain health range and define how you want to behave when the hf drops below or above
that range. Here's the logic of the bot:

- If reflector price update

    1. Fetch all tracked users.
    2. For each tracked user, ensure that the health factor (or `hf` for convenience) is within the user-specied safety ranges.
    3. If the `hf` is below the range:
        
        - Did user choose a conservative strategy?
            - Yes: repay part of the debt (amount (in usdc denom) and asset specified by the user).
            - No: increase collateral (amount (in usdc denom) and asset specified by the user).
    4. If the `hf` is above the range:
        
        - Did user choose a conservative strategy?
            - Yes: increase the user's debt, i.e borrow more (amount (in usdc denom) and asset specified by the user).
            - No: withdraw collateral (amount (in usdc denom) and asset specified by the user).

## Components

There are two main components at work here:

1. The zephyr program: this is the core of the rebalancer. The program executes all of the above-described logics and sends transactions 
that are already built and ready to sign to the client.
2. The listening client. Since I haven't implemented with smart accounts yet, we need a way to sign transactions without users deploying
their secret key to Mercury. The client is a lightweight process that spins up a listener served over `ngrok` for the mercury client
to send the transactions to. *Optionally* the client can also verify the validity of the transaction.  The client can also be used to deploy the zephyr code and to add new users to track or edit parameters for the currently tracked ones.

## Usage

Instructions can be found in the [repository](https://github.com/heytdep/blend-rebalancer/)'s README.

<hr/>

# Why is Zephyr so cool?

You might be asking what makes zephyr so special when creating this type of bot. The first logical answer would be that:

1. Anyone can easily do this.
2. It took me approx a couple of hours to build this (probably less, but can't say with certainty though since I worked with the sdk improvements and fixing a retroshades issue along with it). 

If you think I'm exagerating here, please explain this:

```rust
// START HERE.
fn check_hfs(env: &EnvClient) {
    let tracked: Vec<UserPositionRebalancer> = env.read();
    env.log().debug(format!("tracking {} positions.", tracked.len()), None);

    for pos in tracked {
        let mut pool = BlendPoolWrapper::new(env, pos.pool, MOCKED);
        let user_hf = pool.get_user_hf(env, &pos.p_user);

        env.log().debug(format!("User current hf: {}. Range {}-{}.", user_hf.current, pos.down_lim, pos.up_lim), None);
        let mut message = None;
        
        if user_hf.current > pos.up_lim {
            // User HF is too high, need to increase liabilities or diminish collateral.
            if pos.up_cons {
                // User chose conservative strategy, decreasing collateral.
                message = Some(build_request_object(env, pool, pos.p_user, pos.up_asst, pos.up_amnt, 3));
            } else {
                // User chose non conservative strategy, increasing liabilities.
                message = Some(build_request_object(env, pool, pos.p_user, pos.up_asst, pos.up_amnt, 4));
            }
        } else if user_hf.current < pos.down_lim {
            // User HF is too low, need to increase collateral or repay liabilities.
            if pos.down_cons {
                // User chose conservative strategy, repaying debt.
                message = Some(build_request_object(env, pool, pos.p_user, pos.down_asst, pos.down_amnt, 5));
            } else {
                // User chose non conservative strategy, increasing collateral.
                message = Some(build_request_object(env, pool, pos.p_user, pos.down_asst, pos.down_amnt, 2));
            }
        }
        
        if let Some(message) = message {
            let request = AgnosticRequest {
                body: Some(message),
                url: pos.url,
                method: zephyr_sdk::Method::Post,
                headers: vec![("Content-Type".into(), "application/json".into()), ("Authorization".into(), format!("Basic {}", pos.secret))]
            };
            env.send_web_request(request);
        }
    }
}

fn build_request_object(env: &EnvClient, pool: BlendPoolWrapper, user: String, asset: String, usdc_amount: i64, request_type: u32) -> String {
    let price = pool.get_price(env, &asset);
    let v = usdc_amount as f64 / price;
    let v_1: i128 = (v as i64).try_into().unwrap();
    
    let request = Request {
        request_type,
        address: address_from_str(env, &asset),
        amount: v_1
    };

    build_tx_from_blend_request(env, pool, &user, request)
}

fn build_tx_from_blend_request(env: &EnvClient, pool: BlendPoolWrapper, source: &str, request: Request) -> String {
    let blend_requests: zephyr_sdk::soroban_sdk::Vec<Request> = zephyr_sdk::soroban_sdk::vec![&env.soroban(), request.clone()];
    let args_val: zephyr_sdk::soroban_sdk::Vec<Val> = (
        address_from_str(env, &source),
        address_from_str(env, &source),
        address_from_str(env, &source),
        blend_requests,
    )
        .try_into_val(env.soroban()).unwrap_or(zephyr_sdk::soroban_sdk::Vec::new(&env.soroban()));

    if args_val.len() == 0 {
        return json!({"status": "error", "message": "failed to convert arguments to host val"}).to_string();
    }

    let sequence = {
        let account = stellar_strkey::ed25519::PublicKey::from_string(&source)
            .unwrap()
            .0;

        env.read_account_from_ledger(account)
            .unwrap()
            .unwrap()
            .seq_num as i64
            + 1
    }

    let simulation = env.simulate_contract_call_to_tx(
        source.to_string(),
        sequence,
        pool.as_hash(),
        Symbol::new(env.soroban(), "submit"),
        args_val,
    );

    let mut result = json!({"status": "error", "message": "unknown error during simulation"});
    if let Ok(tx_resp) = simulation {
        let response = tx_resp.tx.unwrap_or("".into());
        result = json!({"status": "success", "envelope": tamper_resources(response), "request_type": request.request_type});
    }

    result.to_string()
}
```

What do you notice? I'll start:

### DB abstraction

> Were is my DB? How are you reading the user's positions?

Zephyr's DB abstraction takes care of everything that is read-written to storage. Here we're simply `env.read()` into our table structure and that's it. Rust macros and generics to their magic.

### Seamless, really, interaction with the chain.

> Wait `let user_hf = pool.get_user_hf(env, &pos.p_user);`??

Yeah, that's exactly why I'm adding protocol specific utils to the SDK. 

**Plus**. Did you know that the protocol utils are all guest-side? And did you know that to add them I just had to paste the blend pool's code and adapt the storage functions to the way we read the ledger from zephyr? This is *insanely* great and makes zephyr an obvious choice to build anything on-chain related. Why does pasting soroban contract code in an offchain service work? You'll have to watch [my Meridian2024 talk](https://heytdep.github.io/post/22/post.html) for that.

> I can build and simulate transactions so simply?

Yes you can. 

```rust
let request = Request {
    request_type,
    address: address_from_str(env, &asset),
    amount: v_1
};

let blend_requests: zephyr_sdk::soroban_sdk::Vec<Request> = zephyr_sdk::soroban_sdk::vec![&env.soroban(), request.clone()];
let args_val: zephyr_sdk::soroban_sdk::Vec<Val> = (
    address_from_str(env, &source),
    address_from_str(env, &source),
    address_from_str(env, &source),
    blend_requests,
)
    .try_into_val(env.soroban()).unwrap_or(zephyr_sdk::soroban_sdk::Vec::new(&env.soroban()));

let sequence = {
    let account = stellar_strkey::ed25519::PublicKey::from_string(&source)
        .unwrap()
        .0;

    env.read_account_from_ledger(account)
        .unwrap()
        .unwrap()
        .seq_num as i64
        + 1
}

let simulation = env.simulate_contract_call_to_tx(
    source.to_string(),
    sequence,
    pool.as_hash(),
    Symbol::new(env.soroban(), "submit"),
    args_val,
);
```

If by now you don't get why this is cool I don't know what to tell you, but if you really need to see more, know that this program can be deployed and started with one bash command using the `mercury-cli` (or using the client's functionality). 

<hr/>

# Next steps

Next up is adding more interactions and protocols to work with. On my todolist I have fxdao (which is partially implemented fwiw) and I still have to figure out if AMMs make sense to add just yet.

You can also expect more interesting stuff, much more than this coming relatively soon. I couldn't be more excited for what's next.

**PS**: I will repeat this in almost every new post: if you're looking to build on soroban DM me ;)
