#![no_main]
use std::collections::HashSet;

use crdt_woot::{crdt::OpSet, test, woot, woot_dumb_impl::WootImpl};
use libfuzzer_sys::fuzz_target;
use rand::Rng;

fuzz_target!(|data: (usize, usize, u64)| { test::test::<WootImpl>(data.2, 10, data.1 % 1000 + 1) });
