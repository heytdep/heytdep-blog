
`2/10/2023`

I recently started learning Zig and as commitment to learning this language, which seems ergonomic and pretty close to Rust
in terms of syntax, I started building a soroban SDK for it. 

For those who don't know it, creating a soroban smart contract without an SDK is non trivial, verbose and requires you to build
conversions and host functions, and even then, helpers of top of those host functions which alone are non-friendly to use. 

In short, you would need to build an sdk for every contract.

That's why I've never seen a soroban contract that doesn't use an SDK. Currently, in the soroban ecosystem there are two SDKs
that allow to build soroban smart contracts:

- the official rust sdk, which is and will probably always be the most ergonomic way of writing a Soroban smart contract.
- the assemblyscript sdk, which is maintained by Soneso (who has an excellent track record for building Stellar sdks).

And hopefully, there will also be a complete Soroban SDK for Zig built by me.

# Progress

This is the first blog about the SDK, which is open source and can be found at https://github.com/heytdep/zig-soroban-sdk/. 
Currently, I've just finished implementing all `Val` conversions for small values (< 56 bits).

# What I've done

This blog series is designed to also help developers that want to build an SDK for all WASM-compilation-compatible languages
as a guide. Building an sdk for Soroban is non-trivial and requires knowledge about how the soroban environment works
and how WebAssembly works too.

I'm going to assume that the reader knows WASM basics, and try to explain at my best what concepts regarding the soroban
environment are needed to build the SDK. 

## `Val` conversions

If you take a look at the WAT code of a soroban smart contract, you'll notice that the exported functions will have all the
same parameter type: `i64`. That's how the soroban virtual machine is going to provide parameters in: `Val`. So, it's clear that
every interaction with the sorboan environment (invocation parameters and calls to host functions) are all based on `Val`. An sdk, should be able to convert those `Val`s (effectively integers) to a the type they represent (as I don't think anyone would like to make operations on a `Val` directly).

To know which type a `Val` represents, you need to look at its tag, and once you're certain that the `Val` represents your type you can go forward and perform all the various bitwise shifts, char matching (for symbols) and host function calls, needed to operate on the `Val` integer as their type (for objects) or directly cast the `Val` to the type for smaller types. 

For example, take this simple contract function that multiplies two `i32`s:

```zig
export fn multiply_i32(a_: Val, b_: Val) Val {
    @setRuntimeSafety(true);
    const a = I32.from_val(a_) catch unreachable;
    const b = I32.from_val(b_) catch unreachable;

    return a.mul(b).to_val();
}
```

As you can see, all of the function's arguments are `Val` (note that `Val` [is a packed struct](https://github.com/heytdep/zig-soroban-sdk/blob/main/src/val.zig#L14)), and then according to the chosen types of the parameter we use the `from_val(val: Val)` method to cast `Val` to a wrapper that then allows to also return a `Val` through `WRAPPER.to_val()`.

In fact, the `multiply_i32()` function is translated in WASM to have the following type:

```wat
(type $t0 (func (param i64 i64) (result i64)))
```

# What's next

Now comes the funniest part: including host functions, working with objects, and creating wrappers that use them for stuff like doing large integer operations, using vectors, writing storage data, requiring auth, etc.
