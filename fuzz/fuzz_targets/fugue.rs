#![no_main]

use crdt_list::{fugue_dumb_impl::FugueImpl, test, test::Action};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: Vec<Action>| { test::test_with_actions::<FugueImpl>(5, 100, data) });
