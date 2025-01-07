use std::env;

fn main() -> std::io::Result<()> {
    let path = env::current_dir()?;
    println!("Current directory: {}", path.display());
    Ok(())
}
