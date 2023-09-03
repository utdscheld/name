mod nma {
    pub mod nma;
}

use nma::nma::assemble;

fn main() {
    println!("Hello, world!");
    assemble("mips_test.asm");
}
