use ndsjit::disasm::try_disasm_arm;
use std::fs;
use std::io::Error;

fn main() -> Result<(), Error> {
    let f = fs::read("gba_bios/gba_bios.bin")?;
    for i in 0..30 {
        for j in 0..4 {
            print!("{:02x?} ", &f[4 * i + j]);
        }
        let res = try_disasm_arm(0, &f[4 * i + j..4 * i + j + 4]);
        println!();
    }
    Ok(())
}
