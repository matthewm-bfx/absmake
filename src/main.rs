use anyhow::Result;
use std::env;
use std::io::BufRead;
use std::io::BufReader;
use std::process::Command;
use std::process::Stdio;

fn main() -> Result<()> {
    // Pass arguments unchanged to Make, except the program name itself
    let args: Vec<String> = env::args().skip(1).collect();

    // Spawn make process
    println!("+ make {}", args.join(" "));
    let mut make_cmd = Command::new("make")
        .args(args)
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start make");

    // Read output
    let output = make_cmd.stdout.as_mut().expect("failed to obtain output");
    let mut reader = BufReader::new(output);
    loop {
        let mut buf = String::new();
        match reader.read_line(&mut buf)? {
            0 => break,
            _ => print!("{buf}"),
        }
    }

    // Wait for completion
    make_cmd
        .wait()
        .expect("failed to wait for process termination");

    Ok(())
}
