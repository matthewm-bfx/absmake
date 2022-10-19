use anyhow::Result;
use regex::Regex;
use std::env;
use std::io::BufRead;
use std::io::BufReader;
use std::process::Command;
use std::process::Stdio;

struct DirectoryTracker {
    current_dir: String,
}

impl DirectoryTracker {
    fn new() -> Self {
        DirectoryTracker {
            current_dir: String::new(),
        }
    }

    fn interpret_line(&mut self, line: &str) {
        let enter_re = Regex::new(r"^make\[[1-9]\]: Entering directory '([^'])'").unwrap();
        let leave_re = Regex::new(r"^make\[[1-9]\]: Leaving directory '([^'])'").unwrap();

        if let Some(caps) = enter_re.captures(line) {
            self.current_dir = caps.get(1).unwrap().as_str().to_owned();
            println!("+ Current dir is now {}", self.current_dir);
        }
    }
}

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

    // Read output stream
    let output = make_cmd.stdout.as_mut().expect("failed to obtain output");
    let mut reader = BufReader::new(output);

    // Parse lines, updating current directory if necessary
    let mut tracker = DirectoryTracker::new();
    loop {
        let mut buf = String::new();
        match reader.read_line(&mut buf)? {
            0 => break,
            _ => tracker.interpret_line(&buf),
        }
        print!("{buf}");
    }

    // Wait for completion
    make_cmd
        .wait()
        .expect("failed to wait for process termination");

    Ok(())
}
