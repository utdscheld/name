mod nma {
    pub mod nma;
}

pub mod u5;
pub mod u6;

use nma::nma::assemble;

fn main() -> Result<(), &'static str> {
    assemble(".artifacts/mips_test.asm", ".artifacts/mips_test.out")?;
    Ok(())
}
