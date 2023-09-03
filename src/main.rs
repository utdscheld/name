mod nma {
    pub mod nma;
}

pub mod u5;
pub mod u6;

use nma::nma::assemble;

fn main() {
    assemble("mips_test.asm");
}
