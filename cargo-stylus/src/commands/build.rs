// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

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
    for contract in args.project.contracts()? {
        let output = contract.build(&config)?;
        log::info!("{:?} contract built: {}", config.target, output.display());
    }
    Ok(())
}
