// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

pub mod reproducible;

use std::{path::PathBuf, process::Stdio};

use cargo_metadata::MetadataCommand;
use escargot::Cargo;

use crate::core::project::contract::Contract;

const WASM_TARGET: &str = "wasm32-unknown-unknown";
const OPT_LEVEL_Z_CONFIG: &str = "profile.release.opt-level='z'";
const UNSTABLE_FLAGS: &[&str] = &[
    "build-std=std,panic_abort",
    "build-std-features=panic_immediate_abort",
];

/// Build target architecture.
#[derive(Clone, Debug, Default)]
pub enum Target {
    /// WebAssembly (Arbitrum Stylus)
    #[default]
    Wasm,
    /// PolkaVM / pallet-revive (RISC-V)
    Pvm,
}

#[derive(Clone, Debug, Default)]
pub struct BuildConfig {
    pub opt_level: OptLevel,
    pub features: Vec<String>,
    pub target: Target,
}

#[derive(Clone, Debug, Default)]
pub enum OptLevel {
    #[default]
    S,
    Z,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("cargo error: {0}")]
    Cargo(#[from] escargot::error::CargoError),
    #[error("cargo metadata error: {0}")]
    CargoMetadata(#[from] cargo_metadata::Error),

    #[error("{0}")]
    Toolchain(#[from] crate::utils::toolchain::ToolchainError),

    #[error("build did not generate wasm file")]
    NoWasmFound,
    #[error("build did not generate ELF file")]
    NoElfFound,
    #[error("failed to execute cargo build")]
    FailedToExecute,
    #[error("cargo build command failed")]
    CargoBuildFailed,
    #[error("polkavm linker error: {0}")]
    PvmLinker(String),
}

pub fn build_contract(contract: &Contract, config: &BuildConfig) -> Result<PathBuf, BuildError> {
    match config.target {
        Target::Wasm => build_contract_wasm(contract, config),
        Target::Pvm => build_contract_pvm(contract, config),
    }
}

fn build_contract_wasm(contract: &Contract, config: &BuildConfig) -> Result<PathBuf, BuildError> {
    info!(@grey, "Building project with Cargo.toml version: {}", contract.version());

    let mut cmd = Cargo::new()
        .args(["build", "--lib", "--locked", "--release"])
        .args(["--target", WASM_TARGET]);
    if !config.features.is_empty() {
        cmd = cmd.args(["--features", &config.features.join(" ")]);
    }
    if !contract.stable() {
        cmd = cmd.args(UNSTABLE_FLAGS.iter().flat_map(|flag| ["-Z", flag]));
    }
    if matches!(config.opt_level, OptLevel::Z) {
        cmd = cmd.args(["--config", OPT_LEVEL_Z_CONFIG]);
    }

    let status = cmd
        .into_command()
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|_| BuildError::FailedToExecute)?;
    if !status.success() {
        return Err(BuildError::CargoBuildFailed);
    }

    let metadata = MetadataCommand::new().exec()?;
    let wasm_path = metadata
        .target_directory
        .join(WASM_TARGET)
        .join("release")
        .join("deps")
        .join(format!("{}.wasm", contract.name()));
    if !wasm_path.exists() {
        return Err(BuildError::NoWasmFound);
    }

    Ok(wasm_path.into())
}

/// Custom RISC-V target specification for PolkaVM (pallet-revive).
const PVM_TARGET_JSON: &str = r#"{
  "arch": "riscv64",
  "cpu": "generic-rv64",
  "crt-objects-fallback": "false",
  "data-layout": "e-m:e-p:64:64-i64:64-i128:128-n32:64-S64",
  "eh-frame-header": false,
  "emit-debug-gdb-scripts": false,
  "features": "+e,+m,+a,+c,+zbb,+auipc-addi-fusion,+ld-add-fusion,+lui-addi-fusion,+xtheadcondmov",
  "linker": "rust-lld",
  "linker-flavor": "ld.lld",
  "llvm-abiname": "lp64e",
  "llvm-target": "riscv64",
  "max-atomic-width": 64,
  "panic-strategy": "abort",
  "relocation-model": "pie",
  "target-pointer-width": 64,
  "singlethread": true,
  "pre-link-args": {
    "ld": [
      "--emit-relocs",
      "--unique",
      "--apply-dynamic-relocs",
      "--no-allow-shlib-undefined",
      "-Bsymbolic"
    ]
  },
  "env": "polkavm",
  "dynamic-linking": true,
  "only-cdylib": true,
  "position-independent-executables": true,
  "static-position-independent-executables": true,
  "relro-level": "full",
  "default-visibility": "hidden",
  "exe-suffix": "",
  "dll-prefix": "",
  "dll-suffix": ".elf"
}"#;

const PVM_TARGET_NAME: &str = "riscv64emac-unknown-none-polkavm";

fn build_contract_pvm(contract: &Contract, config: &BuildConfig) -> Result<PathBuf, BuildError> {
    info!(@grey, "Building PVM contract: {}", contract.name());

    // Write the target JSON to a temp file
    let target_dir = tempfile::tempdir()?;
    let target_json_path = target_dir
        .path()
        .join(format!("{PVM_TARGET_NAME}.json"));
    std::fs::write(&target_json_path, PVM_TARGET_JSON)?;

    let target_json_str = target_json_path.to_str().unwrap();

    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["build", "--lib", "--release"])
        .args(["--target", target_json_str])
        .args(["-Z", "build-std=core,alloc"])
        .args(["-Z", "json-target-spec"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Always enable stylus-sdk/revive for PVM builds so the SDK exports
    // polkavm_derive and enables PVM-specific entrypoint code generation.
    let mut features = config.features.clone();
    let sdk_revive = "stylus-sdk/revive".to_string();
    if !features.contains(&sdk_revive) {
        features.push(sdk_revive);
    }
    cmd.args(["--features", &features.join(",")]);

    if matches!(config.opt_level, OptLevel::Z) {
        cmd.args(["--config", OPT_LEVEL_Z_CONFIG]);
    }

    let status = cmd.status().map_err(|_| BuildError::FailedToExecute)?;
    if !status.success() {
        return Err(BuildError::CargoBuildFailed);
    }

    // Find the ELF output
    // Rust converts hyphens to underscores in output filenames
    let metadata = MetadataCommand::new().exec()?;
    let lib_name = contract.name().replace('-', "_");
    let elf_path = metadata
        .target_directory
        .join(PVM_TARGET_NAME)
        .join("release")
        .join(format!("{lib_name}.elf"));
    if !elf_path.exists() {
        return Err(BuildError::NoElfFound);
    }

    // Link ELF → PolkaVM
    let elf_path: PathBuf = elf_path.into();
    let polkavm_path = elf_path.with_extension("polkavm");
    link_pvm(&elf_path, &polkavm_path)?;

    let size = std::fs::metadata(&polkavm_path)?.len();
    info!(@grey, "Created {} ({} bytes)", polkavm_path.display(), size);

    Ok(polkavm_path.into())
}

#[cfg(feature = "pvm")]
fn link_pvm(elf_path: &std::path::Path, output_path: &std::path::Path) -> Result<(), BuildError> {
    let elf_bytes = std::fs::read(elf_path)?;

    let mut config = polkavm_linker::Config::default();
    config.set_strip(true);
    config.set_optimize(true);

    let linked = polkavm_linker::program_from_elf(
        config,
        polkavm_linker::TargetInstructionSet::ReviveV1,
        &elf_bytes,
    )
    .map_err(|e| BuildError::PvmLinker(e.to_string()))?;

    std::fs::write(output_path, &linked)?;
    Ok(())
}

#[cfg(not(feature = "pvm"))]
fn link_pvm(
    _elf_path: &std::path::Path,
    _output_path: &std::path::Path,
) -> Result<(), BuildError> {
    Err(BuildError::PvmLinker(
        "PVM support requires the 'pvm' feature. Rebuild with --features pvm".to_string(),
    ))
}
