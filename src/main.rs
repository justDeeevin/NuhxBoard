use regex::Regex;
use std::io::{self, BufRead};
use std::process::Command;

fn main() -> io::Result<()> {
    // Run xinput to list devices
    let output = Command::new("xinput").arg("--list").output()?;
    let devices = String::from_utf8_lossy(&output.stdout);

    // Print the list of devices
    println!("{}", devices);

    // You can use xinput test-xi2 --root to capture events
    let mut child = Command::new("xinput")
        .arg("test-xi2")
        .arg("--root")
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    // Read and print input events
    let reader = io::BufReader::new(child.stdout.take().unwrap());
    let mut key_down = false;
    let mut key_up = false;
    for line in reader.lines() {
        let line = line?;
        if line.contains("EVENT type 2") {
            key_down = true;
        } else if line.contains("EVENT type 3") {
            key_up = true;
        } else if line.contains("detail:") {
            let re = Regex::new(r"detail:\s*(\d+)").unwrap();
            if key_down {
                let captures = re.captures(&line).unwrap();
                let keycode: u32 = captures.get(1).unwrap().as_str().parse().unwrap();
                println!("Key down: {}", keycode)
            }
            if key_up {
                let captures = re.captures(&line).unwrap();
                let keycode: u32 = captures.get(1).unwrap().as_str().parse().unwrap();
                println!("Key up: {}", keycode)
            }
            key_down = false;
            key_up = false;
        }
    }

    Ok(())
}
