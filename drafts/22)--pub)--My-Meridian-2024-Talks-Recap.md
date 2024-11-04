`04/11/2024`

TLDR this is the recap of my two Meridian talks. #Meridian2024 was actually the first Meridian I could attend, but the wait for
 meeting irl the folks I worked with for the last years was worth it.

I'll be expanding more on both the talks, likely mainly about the state of decentralization panel, but here's a quick
summary of the follwing talks:

1. [Reusing the Soroban VM and Toolchain for Off-Chain Applications](https://www.youtube.com/watch?v=D115eOIJWXE)
2. [It Takes an Ecosystem: The State of Decentralization on Stellar](https://www.youtube.com/watch?v=P344_wKJshk&list=PLmr3tp_7-7Gh0NyWoqYkBkJC6kwDNqXJh&index=4)

<hr/>

# Reusing the Soroban VM and Toolchain for Off-Chain Applications

This was my first Meridian talk ever. Luckily the topics I talk about are the ones I work with every day
so that might have alleviated the stress. At its core, the talk revolved around showcasing how great the virtual
machine powering Stellar smart contracts is.

I talk about three main use cases and applications that can be powered thanks to the VMs design:

1. Low hanging fruits: exploring the high level svm API.
2. Runtime tooling through VM forks.
3. Using the SVM's host-isolation capabilities to reuse soroban functionality within other VMs.

## SVM High Level API.

Here I suggest that any developer building on Soroban try out the high-level API of the soroban VM by pulling in
the host environemnt's code and playing around with the execution inputs and outputs.

More specifically, I talk about how xBull uses this API to power its balance changes functionality which can give
users more trust before signing transactions (note that it is **not** a bulletproof way of avoiding scams).

## Runtime tooling.

This is the beginning of the advanced topics in the talk. I showcase that by forking the Soroban virtual machine and
adding functionality to the runtime you can achieve incredible tooling such as [Retroshades](https://www.mercurydata.app/products/retroshades).

This happens by pairing the custom runtime along with parallel real-time transaction replay functionality. 

## Reusing the SVM's host.

The was the original main point of the talk and aims to showcase how composable the SVM's architecture is. The design
allows to easily distinguish between the host functionality and the virtual machine, enabling developers to import
the host funcitonality without the VM inside their own VMs and also enabling them to re-use the SDK by tampering
with the module's instantiation process, mainly with the linker, to add all soroban host functions as standard VM
functions.

The only functions that do require some additional manual work are those which depend on reading from webassembly's linear
memory since by default the host environment is bound to use the crate's inner `VmCaller` object. I talk a bit more about
this in the [talk's technical references](https://heytdep.github.io/post/21/post.html).

## Conclusion

Following the talk, the below points should be clear:
1. The SVM is really well developed, among the very best blockchain VMs (not only for the reasons listed in the talk).
2. If you're an audacious developer, trying to work along with the VM might be a good idea.
3. It's even a better idea to rely on the low-level tooling that I built around the idea of this talk. You can learn more
about it on [Mercury's website](https://www.mercurydata.app/).

<hr/>

# State of Decentralization Panel

This wasn't technically a talk, rather a discussion along with Justin (SDF), Anke (SDF) and Alex (Script3) about the past,
present and future about stellar decentralization. There are so many points in this panel which I wouldn't be able to
fit into one article, let alone in a recap but my arguments were mainly that:
1. SCP has a very different consensus than PoS or PoW, it's based on configurable agreements and trust intersections that make
it viable and safe also in the case that only a few organizations are in the tier 1 quorum.
2. Soroban will highly boost community participation from big players and developers in the ecosystem that are not
SDF to contribute to standards, core improvements, running nodes, and other governance related aspects. Consensus is not
the only topic when talking about decentralization!

As I hinted in the beginning of this article, I plan to publish more here about the state of decentralization and
improvement proposals, so this is just the beginning.
