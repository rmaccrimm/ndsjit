This is an early work in progress ARM cpu emulator with the hopes of one day being a functioning
Nintendo DS emulator. 

The primary goal for this project is to learn about and implement a functioning CPU emulator using
just-in-time recompilation (also called dynamic binary translation, among other things) to translate
code from ARM machine code to x86_64 machine code at runtime. It's also provided a great excuse 
to learn Rust!

## Status
- The emitter (asm::x64::EmitterX64) can emit and run most forms of the x86_64 MOV instruction, 
  as well as POP/PUSH
- Have started decoding LDR and STR instructions (disasm::armv4t)

## Building and Testing
Currently supports only 64-bit Window (with plans to target Linux as well). The project is built 
with Cargo using the `cargo build` command, and unit tests are run with `cargo test`.