#[cfg(feature = "diagnostics")]
use sysinfo::{System, Pid, ProcessesToUpdate};

#[cfg(feature = "diagnostics")]
pub(crate) fn print_status(node_count: usize) {
    use std::io::Write;

    let pid = Pid::from(std::process::id() as usize);
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), false);
    let ram_kb = sys.process(pid)
        .map(|p| p.memory() / 1024)
        .unwrap_or(0);

    print!("\rnodes: {:>8} | RAM: {:>6} KB ", node_count, ram_kb);
    std::io::stdout().flush().unwrap();
}