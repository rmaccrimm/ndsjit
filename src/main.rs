use std::io::Error;
use ndsjit::cpu::ARM7;
// When retrieving code to run for a given address there are 3 possibilities
// Compiled, uncompiled, and uncompileable/requires interpretation

/// Expecting that we will compile instructions until a branch instruction is reached that requires
/// runtime information, which will be interpreted
// struct TranslationResult {
    // pub compiled_code: Option<ExecBuffer>,
    // pub branch_instr: Opcode,
// }

// fn translate_block(cpu: &ARM7) -> TranslationResult {
//     let mut pc = cpu.get_pc() as usize;
//     let mut asm = AssemblerX64::new(RegAllocation::default());
//     let branch_instr = loop {
//         // Note - not using read_word, because we don't actually want to trigger any side effects
//         // here
//         let bytes = u32::from_le_bytes(cpu.mem[pc..pc + 4].try_into().unwrap());
//         let instr = try_disasm_arm(pc as u32, bytes).unwrap();
//         if instr.requires_interpreter() {
//             break instr;
//         } else {
//             asm.assemble(instr);
//         }
//         pc += 4;
//     };
//     TranslationResult {
//         compiled_code: asm.get_exec_buffer(),
//         branch_instr,
//     }
// }

// fn interpret_instr(cpu: &mut ARM7, instr: Opcode) {
    // println!(instr);
// }

fn main() -> Result<(), Error> {
    let mut cpu = ARM7::new();
    cpu.load_bios("gba_bios.bin")?;

    // let mut cache = HashMap::new();

    // for _ in [..1] {
    //     let pc = cpu.get_pc();
    //     if !cache.contains_key(&pc) {
    //         cache.insert(pc, translate_block(&cpu));
    //     }
    //     let result = cache.get(&pc).unwrap();
    //     if let Some(f) = &result.compiled_code {
    //         f.call(cpu.vreg_base_ptr(), cpu.mem_base_ptr());
    //     }
    //     interpret_instr(&mut cpu, result.branch_instr);
    // }
    Ok(())
}
