use std::io::Error;
use ndsjit::cpu::ARM7;

fn main() -> Result<(), Error> {
    let mut cpu = ARM7::new();
    cpu.load_bios("gba_bios.bin")?;
    Ok(())
}
