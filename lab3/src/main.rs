use crate::fuzzer::{start_fuzzer};

mod fuzzer;

fn main() -> Result<(), String> {
    env_logger::init();

    let before = "./grammars/grammar.cfg";
    let after1 = "./grammars/intersections/ll_approx_inter.cfg";
    let after2 = "./grammars/intersections/lr_approx_inter.cfg";
    start_fuzzer(before, after1, after2);
    Ok(())
}