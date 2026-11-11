use std::io;

use rite::{compiler::Compiler, runtime::Runtime};

fn main() -> io::Result<()> {
    let mut compiler = Compiler::new();
    compiler.add_module("test", "test.ri")?;
    compiler.add_module("std", "std")?;

    let Ok(expr) = compiler.lower() else {
        return Ok(());
    };

    println!("{expr:?}");

    let mut runtime = Runtime::new();
    let output = runtime.run(&expr);

    println!("{output:?}");

    Ok(())
}
