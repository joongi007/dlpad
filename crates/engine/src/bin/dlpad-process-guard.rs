//! Development helper; the GUI also registers its own headless entry.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    dlpad_engine::process::guardian_entry()?;
    eprintln!("dlpad guardian: internal invocation required");
    std::process::exit(2);
}
