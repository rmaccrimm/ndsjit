use std::{
    error::Error,
    io::{BufRead, BufReader, Write},
    process::{ChildStdin, Command, Stdio},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut proc = Command::new("docker")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args([
            "run",
            "--rm",
            "-i",
            "--log-driver=none",
            "-a",
            "stdin",
            "-a",
            "stdout",
            "-a",
            "stderr",
            "asm",
        ])
        .spawn()?;

    let input = "LDR r0,[r1,r2,LSL#30]!\nANDr1,r2,r3\n";

    let child_in = proc.stdin.as_mut().unwrap();
    child_in.write_all(&input.as_bytes()).unwrap();
    proc.wait().expect("process failed");

    let child_out = BufReader::new(proc.stdout.as_mut().unwrap());
    // let mut child_err = BufReader::new(proc.stderr.as_mut().unwrap());

    // let mut line_out = String::new();

    for line in child_out.lines() {
        println!("{}", line.unwrap());
    }
    Ok(())
}
