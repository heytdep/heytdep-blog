`15/10/2024`

If you've made it to this post, it likely means that you are interested in the more technical aspect of my [Meridian 2024](https://meridian.stellar.org) talk about the soroban virtual machine.

As promised in the talk, this document serves the purpose of filling in the technical blanks that cannot be talked about in the actual talk due to accessibility and mainly available time.

**Talk title**: Reusing the Soroban VM and Toolchain for Off-Chain Applications
**Description**: You may have thought that Soroban was only designed to run on-chain, but this session is going to let you in on a secret: the environment SDKs and tooling hold hidden potential beyond on-chain execution. This talk dives deep into these exciting off-chain use cases and explores how we can unlock their power. Join Co-Founder of Xycloo Labs, Tommaso De Ponti, to learn more about what’s powering the next generation of smart contracts on Stellar.

# Behind Retroshades: The Retroshades SVM Fork.

## Why Forking the SVM is Not Complex

The Soroban virtual machine is particularly well suited to be forked because of the level of abstraction from the VM logic and related functionality in the host context. As in itself, thanks to its architecture and metaprogramming, there are little references in the host context implementation towards the actual virtual machine logic (hf signatures, memory read/writes, wasm -> soroban types casts, etc) which makes writing additional logic, especially if similar to existing one, incredibly easier than on other blockchain VMs.

## Adding Off-Chain Functionality.

When talking about forking the soroban virtual machine, there are a few things to keep into account. First of all, you need to understand whether the customization you aim to add involves the need of a dedicated soroban host function. If so, you'll need to tamper with the `env.json` host functions definition to create an export within a module. This will automatically add the function definition into the `VmCaller` trait, where you'll find all other Soroban host functions:

```rust
impl VmCallerEnv for Host {
    type VmUserState = Host;

    fn zephyr_emit(
        &self,
        _vmcaller: &mut VmCaller<Host>,
        target: Val,
        event: Val,
    ) -> Result<Void, HostError> {
        self.record_retroshade(target, event)?;
        Ok(Val::VOID)
    }
}
```

### Storing within the store context.

If your fork requires to keep data in-memory within the wasm store context of the VM so that it can be accessed elsewhere either during runtime by other host functions or at the end of the execute as an effect, you'll have to add your data to the `HostImpl` object.

```rust
#[derive(Clone, Default)]
struct HostImpl {
    zephyr_adapter: RefCell<ZephyrAdapter>,
// ...
}
```

<hr/>

# VM Abstraction: Behing the ZVM's Soroban Functionality.

Part of the talk revolves around virtual machine abstraction and why it's needed to implement soroban host functionality within other WASM VMs. This is an incredibly interesting use case because it allows you to:
- Actuall check out performance on wasm compilers vs interpreters, both with cached and non-cached binaries.
- Use soroban's host-guest object transmission to power other unrelated VMs (blockchain VMs, baremetal VMs).

This looks straightforward, but in reality there are a few facets to be taken into account.

## Most of the Soroban VM is VM-abstraction friendly.

Again, the SVM is very well equipped when it comes to VM abstraction (see paragraphs above). Most of Soroban's host function don't require a handle over the VM's memory, and for those you'll just have to find a good way for them to to be accessed from the guest. You'll also need to think about keeping a soroban host environment within your VM's context during its execution.

The best way to achieve this is probably to wrap the soroban host within your own host implementation:

```rust
/// Zephyr Host State Implementation.
#[derive(Clone)]
pub struct HostImpl<DB: ZephyrDatabase, L: LedgerStateRead> {
// ...
    /// Wrapper for the Soroban Host Environment
    pub soroban: RefCell<soroban_env_host::Host>,
}
```

And modify Soroban's linking macros to unwrap the actual soroban host from the host in store:

```rust
macro_rules! generate_dispatch_functions {
    {
        $(
            $(#[$mod_attr:meta])*
            mod $mod_name:ident $mod_str:literal
            {
                $(
                    $(#[$fn_attr:meta])*
                    { $fn_str:literal, $($min_proto:literal)?, $($max_proto:literal)?, fn $fn_id:ident ($($arg:ident:$type:ty),*) -> $ret:ty }
                )*
            }
        )*
    }
    =>
    {
        $(
            $(
                $(#[$fn_attr])*
                pub(crate) fn $fn_id<DB: ZephyrDatabase + Clone + 'static, L: LedgerStateRead + 'static>(caller: wasmi::Caller<Host<DB, L>>, $($arg:i64),*) ->
                    (i64,)
                {
                    let host: soroban_env_host::Host = Host::<DB, L>::soroban_host(&caller);
                }
            )*
        )*
    };
}
```

That said, functions that do need to read from the VM's memory need to be rewired to accept a VM generic instead of the VmCaller object.

To achieve this we need to

- Create a generic VM object trait that can read/write to the VM's memory.
- Implement the changes into existing host environment functions that read/write the VMs memory.

```rust
/// A trait that VMs that want to work with a custom context should
/// implement.
pub trait CustomContextVM {
    /// Return WASMI's Memory handle.
    fn read(&self, mem_pos: usize, buf: &mut [u8]);

    fn data(&self) -> &[u8];

    fn write(&mut self, pos: u32, slice: &[u8]) -> i64;

    fn data_mut(&mut self) -> &mut [u8];
}
```

```rust
pub fn vec_new_from_linear_memory_mem<M: CustomContextVM>(
    &self,
    m: M,
    vals_pos: U32Val,
    len: U32Val,
) -> Result<VecObject, HostError> {
    let MemFnArgsCustomVm { pos, len, .. } = self.get_mem_fn_args_custom_vm(&m, vals_pos, len);
    Vec::<Val>::charge_bulk_init_cpy(len as u64, self)?;
    let mut vals: Vec<Val> = vec![Val::VOID.to_val(); len as usize];
    // charge for conversion from bytes to `Val`s
    self.charge_budget(
        ContractCostType::MemCpy,
        Some((len as u64).saturating_mul(8)),
    )?;
    self.metered_vm_read_vals_from_linear_memory_mem::<8, Val, M>(
        &m,
        pos,
        vals.as_mut_slice(),
        |buf| Ok(Val::from_payload(u64::from_le_bytes(*buf))),
    )?;

    for v in vals.iter() {
        self.check_val_integrity(*v)?;
    }
    self.add_host_object(HostVec::from_vec(vals)?)
}
```

```rust
pub(crate) fn metered_vm_read_vals_from_linear_memory_mem<
    const VAL_SZ: usize,
    VAL,
    M: CustomContextVM,
>(
    &self,
    m: &M,
    mem_pos: u32,
    buf: &mut [VAL],
    from_le_bytes: impl Fn(&[u8; VAL_SZ]) -> Result<VAL, HostError>,
) -> Result<(), HostError> {
    // ...
    let mem_data = m.data();
    let mem_slice = mem_data
        .get(mem_range)
        .ok_or_else(|| self.err_oob_linear_memory())?;

    // ...

    Ok(())
}
```
