use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{address, hex, AccountInfo, Bytecode, TransactTo, U256},
};
use revmc_examples_runner::build_evm;
use std::hint::black_box;

include!("./common.rs");

fn main() {
    let num =
        std::env::args().nth(1).map(|s| s.parse().unwrap()).unwrap_or_else(|| U256::from(100));
    // The bytecode runs fib(input + 1), so we need to subtract 1.
    let actual_num = num.saturating_sub(U256::from(1));

    let db = CacheDB::new(EmptyDB::new());
    let mut evm = build_evm(db);
    let fibonacci_address = address!("0000000000000000000000000000000000001234");
    evm.db_mut().insert_account_info(
        fibonacci_address,
        AccountInfo {
            code_hash: FIBONACCI_HASH.into(),
            code: Some(Bytecode::new_raw(FIBONACCI_CODE.into())),
            ..Default::default()
        },
    );
    evm.context.evm.env.tx.transact_to = TransactTo::Call(fibonacci_address);
    println!("actual_num.to_be_bytes_vec() {:?}", hex::encode(actual_num.to_be_bytes_vec()));
    evm.context.evm.env.tx.data = actual_num.to_be_bytes_vec().into();
    let result = evm.transact().unwrap();
    eprintln!("{:#?}", result.result);

    println!("fib({num}) = {}", U256::from_be_slice(result.result.output().unwrap()));

    bench(100, "fibonacci", || {
        evm.transact().unwrap();
    });
}

fn bench<T>(n_iters: u64, name: &str, mut f: impl FnMut() -> T) {
    let warmup = (n_iters / 10).max(10);
    for _ in 0..warmup {
        black_box(f());
    }

    let t = std::time::Instant::now();
    for _ in 0..n_iters {
        black_box(f());
    }
    let d = t.elapsed();
    eprintln!("{name}: {:>9?} ({d:>12?} / {n_iters})", d / n_iters as u32);
}
