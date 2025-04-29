fn main() {
    // read env variables that were set in build script
    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");

    // choose whether to start the UEFI or BIOS image
    let uefi = false;

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    if uefi {
        // FIXME: This is the old interface, lookup how to call `ovmf_prebuilt` now.
        // cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive")
            .arg(format!("format=raw,file={uefi_path}"));
    } else {
        cmd
            // Use this to change the RAM size
            // .args(["-m", "500M"])
            // Use this to write the OS output to a log file
            // .args(["-serial", "file:serial.log"])
            .args(["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"])
            .args(["-serial", "stdio"])
            .args(["-drive", &format!("format=raw,file={bios_path}")]);
    }
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
