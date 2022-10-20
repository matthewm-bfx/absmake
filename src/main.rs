use anyhow::Result;
use regex::Regex;
use std::env;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

struct LineProcessor {
    current_dir: String,
}

impl LineProcessor {
    // Default constructor
    fn new() -> Self {
        LineProcessor {
            current_dir: String::new(),
        }
    }

    // Process and possibly print a line from Make
    fn process_line(&mut self, line: &str) {
        let enter_re = Regex::new(r"^make\[[1-9]\]: Entering directory '([^']+)'").unwrap();
        let leave_re = Regex::new(r"^make\[[1-9]\]: Leaving directory '([^']+)'").unwrap();

        // Handle entering a directory
        if let Some(caps) = enter_re.captures(line) {
            self.current_dir = caps.get(1).unwrap().as_str().to_owned();
        }
        // Handle leaving a directory. We can only leave a directory if we are already in it.
        else if let Some(caps) = leave_re.captures(line) {
            let left_dir = caps.get(1).unwrap().as_str().to_owned();
            if left_dir == self.current_dir {
                let path = PathBuf::from(left_dir);
                let parent = path.parent().expect("failed to identify parent path");
                self.current_dir = parent.to_str().unwrap().to_owned();
            }
        }

        // Print out the line to console
        print!("{line}");
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
    let mut proc = LineProcessor::new();
    loop {
        let mut buf = String::new();
        match reader.read_line(&mut buf)? {
            0 => break,
            _ => proc.process_line(&buf),
        }
    }

    // Wait for completion
    make_cmd
        .wait()
        .expect("failed to wait for process termination");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpret_line() {
        let mut tracker = LineProcessor::new();
        assert_eq!(tracker.current_dir, "");

        tracker.process_line("random line");
        assert_eq!(tracker.current_dir, "");

        // Enter directory
        let enter_line = r"make[1]: Entering directory '/home/me/source/main'";
        tracker.process_line(enter_line);
        assert_eq!(tracker.current_dir, "/home/me/source/main");

        // Leave directory
        let leave_line = r"make[1]: Leaving directory '/home/me/source/main'";
        tracker.process_line(leave_line);
        assert_eq!(tracker.current_dir, "/home/me/source");

        // Leave directory only works if we are already in the directory
        let leave_line = r"make[1]: Leaving directory '/home/me/something/else";
        tracker.process_line(leave_line);
        assert_eq!(tracker.current_dir, "/home/me/source");
    }
}
