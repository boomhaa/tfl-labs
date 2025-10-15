use std::thread;
use crate::tests_helper::fuzz_tests::start_fuzzer;
use crate::tests_helper::meta_tests::start_meta_tests;
use crate::utils::rules_additioner::start_rules_additioner;

mod utils;
mod tests_helper;

fn main() {
    env_logger::init();
    //let stack_size = 1024 * 1024 * 1024;
    //let builder = thread::Builder::new().stack_size(stack_size);
//
    //let handler = builder.spawn(|| {
    //    start_rules_additioner();
    //}).unwrap();
    //handler.join().unwrap();

    start_fuzzer();
    start_meta_tests();
}
