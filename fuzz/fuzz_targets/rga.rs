#![no_main]

use crdt_list::{rga_dumb_impl::RgaImpl, test, test::Action};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: Vec<Action>| { test::test_with_actions::<RgaImpl>(5, 100, data) });
