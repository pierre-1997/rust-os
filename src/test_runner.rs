use std::path::PathBuf;

use regex::Regex;

fn main() {
    // read env variables that were set in build script
    let mut build_cmd = std::process::Command::new("cargo");
    build_cmd
        .arg("test")
        .args(["-p", "kernel"])
        .arg("--no-run")
        .args(["--target", "x86_64-unknown-none"]);

    let cmd_output = build_cmd.output().unwrap();
    let cmd_output = str::from_utf8(&cmd_output.stderr).unwrap();

    let re = Regex::new(r"Executable unittests.*\(([^)]+)\)").unwrap();
    let bin_path = PathBuf::from(&re.captures(cmd_output).unwrap()[1]);

    // create a BIOS disk image
    // set by cargo, build scripts should use this directory for output files
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let bios_path = out_dir.join("bios.img");
    bootloader::BiosBoot::new(&bin_path)
        .create_disk_image(&bios_path)
        .unwrap();

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd
        // Use this to change the RAM size
        // .args(["-m", "500M"])
        // Use this to write the OS output to a log file
        // .args(["-serial", "file:serial.log"])
        .args(["-serial", "stdio"])
        .args([
            "-drive",
            &format!("format=raw,file={}", bios_path.display()),
        ]);

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
