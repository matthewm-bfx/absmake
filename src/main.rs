use anyhow::Result;
use regex::Regex;
use std::env;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

struct LineProcessor {
    // Current state
    current_dir: String,

    // Regexes for line matching
    enter_re: Regex,
    leave_re: Regex,
    error_re: Regex,
}

impl LineProcessor {
    // Default constructor
    fn new() -> Self {
        LineProcessor {
            current_dir: String::new(),
            enter_re: Regex::new(r"^make\[[1-9]\]: Entering directory '([^']+)'").unwrap(),
            leave_re: Regex::new(r"^make\[[1-9]\]: Leaving directory '([^']+)'").unwrap(),
            error_re: Regex::new(r"^[^/][^:]+:[0-9]+:[0-9]+: (error|warning|note):").unwrap(),
        }
    }

    // Process and possibly return a line from Make
    fn process_line(&mut self, line: &str) -> Option<String> {
        // Handle entering a directory
        let trimmed = line.trim_end();
        if let Some(caps) = self.enter_re.captures(trimmed) {
            self.current_dir = caps.get(1).unwrap().as_str().to_owned();
            return Some(format!("{trimmed}"));
        }
        // Handle leaving a directory. We can only leave a directory if we are already in it.
        else if let Some(caps) = self.leave_re.captures(trimmed) {
            let left_dir = caps.get(1).unwrap().as_str().to_owned();
            if left_dir == self.current_dir {
                let path = PathBuf::from(left_dir);
                let parent = path.parent().expect("failed to identify parent path");
                self.current_dir = parent.to_str().unwrap().to_owned();
                return Some(format!("{trimmed}"));
            }
        }
        // Add path to a diagnostic message
        else if self.error_re.is_match(trimmed) {
            return Some(format!("{}/{trimmed}", self.current_dir));
        }
        // Anything else, just pass through
        else {
            return Some(format!("{trimmed}"));
        }

        None
    }

    // Convenience method to print the result of prcoess_line()
    fn print_line(&mut self, line: &str) {
        if let Some(line) = self.process_line(line) {
            println!("{line}");
        }
    }
}

fn main() -> Result<()> {
    // Pass arguments unchanged to Make, except the program name itself
    let args: Vec<String> = env::args().skip(1).collect();

    // Spawn make process
    println!("+ make {}", args.join(" "));
    let mut make_cmd = Command::new("sh")
        .arg("-c")
        .arg(format!("make {} 2>&1", args.join(" ")))
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
            _ => proc.print_line(&buf),
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

        // Add path to a diagnostic message
        let diag_line = r"ui/mainform.cpp:32:5: error: syntax error";
        let output = tracker.process_line(diag_line).expect("missing output line");
        assert_eq!(output, "/home/me/source/ui/mainform.cpp:32:5: error: syntax error");
    }
}
