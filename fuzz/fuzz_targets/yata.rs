#![no_main]

use crdt_woot::{test, test::Action, yata_dumb_impl::YataImpl};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: Vec<Action>| { test::test_with_actions::<YataImpl>(5, &data) });
