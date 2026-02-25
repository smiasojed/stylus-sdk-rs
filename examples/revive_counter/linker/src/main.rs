use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: revive-linker <input.elf> [output.polkavm]");
        process::exit(1);
    }

    let input = &args[1];
    let output = if args.len() > 2 {
        args[2].clone()
    } else {
        input.replace(".elf", ".polkavm")
    };

    let elf_bytes = fs::read(input).unwrap_or_else(|e| {
        eprintln!("Failed to read {input}: {e}");
        process::exit(1);
    });

    let mut config = polkavm_linker::Config::default();
    config.set_strip(true);
    config.set_optimize(true);

    let linked = polkavm_linker::program_from_elf(
        config,
        polkavm_linker::TargetInstructionSet::ReviveV1,
        &elf_bytes,
    )
    .unwrap_or_else(|e| {
        eprintln!("Failed to link: {e}");
        process::exit(1);
    });

    fs::write(&output, &linked).unwrap_or_else(|e| {
        eprintln!("Failed to write {output}: {e}");
        process::exit(1);
    });

    eprintln!("Created {output} ({} bytes)", linked.len());
}
