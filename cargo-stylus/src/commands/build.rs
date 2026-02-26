// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use stylus_tools::core::build::Target;

use crate::{
    common_args::{BuildArgs, ProjectArgs},
    error::CargoStylusResult,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(flatten)]
    build: BuildArgs,
    #[command(flatten)]
    project: ProjectArgs,
}

pub fn exec(args: Args) -> CargoStylusResult {
    let config = args.build.config();
    match config.target {
        Target::Pvm => exec_pvm(&config),
        Target::Wasm => {
            for contract in args.project.contracts()? {
                let _wasm_path = contract.build(&config)?;
            }
            Ok(())
        }
    }
}

fn exec_pvm(config: &stylus_tools::core::build::BuildConfig) -> CargoStylusResult {
    use cargo_metadata::MetadataCommand;
    use stylus_tools::core::project::contract::Contract;

    // For PVM builds, discover the current package directly via cargo metadata,
    // bypassing the Stylus.toml workspace requirement.
    let metadata = MetadataCommand::new().no_deps().exec()?;

    // Find the root package
    let root_id = metadata
        .resolve
        .as_ref()
        .and_then(|r| r.root.as_ref());

    let package = if let Some(root_id) = root_id {
        metadata.packages.iter().find(|p| &p.id == root_id)
    } else if metadata.packages.len() == 1 {
        metadata.packages.first()
    } else {
        None
    };

    let package = package.ok_or_else(|| {
        eyre::eyre!(
            "Could not determine which package to build. \
             Run from within a contract directory."
        )
    })?;

    let contract = Contract::try_from(package)?;
    let polkavm_path = contract.build(config)?;
    log::info!("PVM contract built: {}", polkavm_path.display());
    Ok(())
}
