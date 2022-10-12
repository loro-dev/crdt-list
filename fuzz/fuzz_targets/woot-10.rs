#![no_main]

use crdt_list::{test, test::Action, woot_dumb_impl::WootImpl};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: Vec<Action>| { test::test_with_actions::<WootImpl>(10, 100, data) });
