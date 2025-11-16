use crate::fuzzer::start_fuzzer;

mod fuzzer;

fn main() {
    env_logger::init();
    start_fuzzer();

}
