`06/11/2024`

TLDR I just opened a core PR to add opt-in reentrancy to Soroban. Calling all developers to discuss this!

# Back Story and Proposal

I kind of always gave for granted that Soroban would have always had reentrancy disabled. I thought,
it's better for security. And it is. I built xycLoans, Soroban's very first flash loans protocol with this in mind
and with approval-based flash loans. I never questioned having to add reentrancy before this morning after seeing
this comment on the Stellar developers discord server:

<img src="/images/discord-screenshot.png"/>

Previously this week, I've also been digging a bit into some other blockchain VMs code, including Arbitrum's multiVM, or better
Arbitrum's stylus SDK since their VM is heavily memory based, and they also do have opt-in reentrancy. These inputs were enough
for me to pursue a fairly low hanging fruit in my free time this afternoon. In fact, I already knew where to look and how to go
about implementing this. Ironically, crafting the tests took longer than the actual change!

The proposal's core is to add two new host functions:

1. `call_reentrant`
2. `try_call_reentrant`

Both with the same signatures as `call` and `try_call`. You can learn more about my reasoning behind this design [on the PR itself](https://github.com/stellar/rs-soroban-env/pull/1491), but the main points are that 

> this makes the change backwards compatible.

> this way it is trivial to discover which contracts do use reentrant function calls (because the module imports the reentrant calls,
 this is hardcoded in the binary and not runtime-derived), instructing developers calling them to be extra careful with 
 including them in their contract's logic.

> adding other host functions is not an efficiency burden thanks to the newly added streamlined linking.

<hr/>

# Why and Join the Discussion!

There a few reasons why we'd want opt-in reentrancy:

- Some DeFi fundamentals are simply better off with reentrancy.
- Easier for EVM devs that are used to working with reentrancy to adapt and potentially port their codebases.
- Makes the life of solidity > soroban wasm compilers like solang much easier when it comes to porting existing evm contracts. 
- Opt-in with proper safety and discoverability about the risks is a win-win in my opinion. 

But I'd love for other developers and people in the community to share their thoughts on either the implementation
or the motivation. [I've talked about this before](https://www.youtube.com/watch?v=P344_wKJshk&list=PLmr3tp_7-7Gh0NyWoqYkBkJC6kwDNqXJh&index=4), if community developers (and also the broader community) get more
involved with the network's fundamentals that aligns the development in the right direction.
