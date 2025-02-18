use anyhow::{Result, Context};

fn do_something() -> Result<()> {
    let value = std::fs::read_to_string("file.txt")
        .context("Failed to read file.txt")?;  // 这里使用了 `?` 操作符和上下文信息
    println!("File contents: {}", value);
    Ok(())
}

fn main() -> Result<()> {
    do_something().context("Something went wrong in the main function")?;
    Ok(())
}

