use std;
use std::process;

pub fn read_pin(id: &str) -> u8 {
    let mut cmd = process::Command::new("gpio");
    cmd.arg("read").arg(id);
    let hi = String::from_utf8_lossy(&cmd.output().unwrap().stdout).trim() == "1";
    if hi {
        return 1;
    } else {
        return 0;
    }
}
