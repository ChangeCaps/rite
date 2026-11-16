use std::{
    fs, io,
    process::{Command, Stdio},
};

use rite::{chez, compiler::Compiler};

fn main() -> io::Result<()> {
    let mut compiler = Compiler::new();
    compiler.add_module("std", "std", false)?;
    compiler.add_module("rite", "rite", true)?;
    // compiler.add_module("test", "test.ri", true)?;

    let Ok(expr) = compiler.lower() else {
        return Ok(());
    };

    let output = chez::codegen(&expr);
    fs::write("out.ss", &output)?;

    Command::new("scheme")
        .arg("--optimize-level")
        .arg("3")
        .arg("--script")
        .arg("out.ss")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    Ok(())
}
