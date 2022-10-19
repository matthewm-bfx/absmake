use std::env;
use std::process;
use std::process::Stdio;
use anyhow::Result;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Spawn make process
    let mut make_cmd = process::Command::new("make")
        .args(args)
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start make");

    // Read output

    // Wait for completion
    make_cmd.wait().expect("failed to wait for process termination");

    Ok(())
}
