For those who do not know it, I've been working on a rust crate that allows for custom ingestion from a stellar core captive process. The crate is [rs-ingest](https://github.com/xycloo/rs-ingest), and even though the library is still unsafe, I tried stress testing it by catching up (almost, when talking big numbers) the whole Stellar Mainnet history, precisely up to ledger `47803575`. 

# TLDR;

The process took 1344779.26723297 seconds, or ~15.5 days running on a `m5.xlarge` AWS ec2 instance running `ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-20230516` and with 500gb of storage (most of the storage was not used as we are not saving the caught up data, just going thorugh it. But take into account you'll need at least ~30GB of free space for the checkpoint buckets catchup processes continuously).

Note that the ledgers of the first years of the network (2015-2018/19) will take much less time to process as more recent ledgers are much fuller than old ones. 

# Setting up the process

rs-ingest was born with the intent of working on Futurenet, but next xdr is compatible with the current mainnet xdr and so is the futurenet-enabled core. 

I also choose (unwisely) to only fetch from one archive meaning that I likely have not processed about 10 checkpoints (~640 ledgers). 

Anyways, the implementation of this test is very simple as it relies on rs-ingest:

```rust
use ingest::{IngestionConfig, CaptiveCore, Range, BoundedRange, SupportedNetwork, BufReaderError};
use stellar_xdr::next::LedgerCloseMeta;
use std::{time::{Instant, Duration}, fs::File};
use std::io::{Write, Result};


fn write_to_checkpoint_file(duration: Duration) -> Result<()> {
    let checkpoint_path = "checkpoint_duration.txt";

    let mut file = File::create(checkpoint_path)?;
    write!(file, "{:?}", duration)?;

    Ok(())
}

mod rs_ingest_test {

    pub fn multi_thread_read_ledgers_range_and_drop(captive_core: &mut CaptiveCore, start: u32, end: u32) {
        let range = Range::Bounded(BoundedRange(start, end));
        
        let rx = captive_core.prepare_ledgers_multi_thread(&range).unwrap();

        let mut last: u32 = 0;

        for ledger in rx.iter() {
            if let Some(meta) = ledger.ledger_close_meta {
                let ledger_seq = match meta.ledger_close_meta {
                    LedgerCloseMeta::V1(v1) => v1.ledger_header.header,
                    LedgerCloseMeta::V0(v0) => v0.ledger_header.header,
                    LedgerCloseMeta::V2(v2) => v2.ledger_header.header,
                };

                last = ledger_seq.ledger_seq;
            }
        }
    }
}

let config = IngestionConfig {
    executable_path: "/usr/local/bin/stellar-core".to_string(),
    context_path: Default::default(),
    network: SupportedNetwork::Pubnet,
    bounded_buffer_size: None,
    staggered: None
};

let mut captive_core = CaptiveCore::new(config);
rs_ingest_test::multi_thread_read_ledgers_range_and_drop(&mut captive_core, start, end);

let elapsed_time = start_time.elapsed();
write_to_checkpoint_file(elapsed_time).unwrap();

```

> This code assumes you have a compatible `stellar-core` in `/usr/local/bin/stellar-core`.

# Improving

Besides some improvements in the library itself, catching up could be potentially be sped up and improved in terms of reporting checkpoints by interacting with archivers directly from our rust code. I'm starting to research on writing such crate, which would allow for more customization during catchups and to catchup while running a live captive core process on the same instance. 
