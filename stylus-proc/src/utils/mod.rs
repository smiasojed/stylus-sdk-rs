// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Macro generation utilities.

use sha3::{Digest, Keccak256};
use syn::{punctuated::Punctuated, Token};
use syn_solidity::SolIdent;

pub mod attrs;

#[cfg(test)]
pub mod testing;

/// Check if a name is a Solidity keyword.
pub fn is_sol_keyword(name: &str) -> bool {
    lazy_static::lazy_static! {
        static ref UINT_REGEX: regex::Regex = regex::Regex::new(r"^uint(\d+)$").unwrap();
        static ref INT_REGEX: regex::Regex = regex::Regex::new(r"^int(\d+)$").unwrap();
        static ref BYTES_REGEX: regex::Regex = regex::Regex::new(r"^bytes(\d+)$").unwrap();
    }

    if let Some(caps) = UINT_REGEX.captures(name) {
        let bits: usize = caps[1].parse().unwrap();
        if bits % 8 == 0 {
            return true;
        }
    }
    if let Some(caps) = INT_REGEX.captures(name) {
        let bits: usize = caps[1].parse().unwrap();
        if bits % 8 == 0 {
            return true;
        }
    }
    if let Some(caps) = BYTES_REGEX.captures(name) {
        let n: usize = caps[1].parse().unwrap();
        if n <= 32 {
            return true;
        }
    }
    matches!(
        name,
        "address" | "bytes" | "bool" | "int" | "uint"
            | "is" | "contract" | "interface"
            | "after" | "alias" | "apply" | "auto" | "byte" | "case" | "copyof"
            | "default" | "define" | "final" | "implements" | "in" | "inline"
            | "let" | "macro" | "match" | "mutable" | "null" | "of" | "partial"
            | "promise" | "reference" | "relocatable" | "sealed" | "sizeof"
            | "static" | "supports" | "switch" | "typedef" | "typeof" | "var"
    )
}

pub fn get_generics(
    generics: &syn::Generics,
) -> (
    Punctuated<syn::GenericParam, Token![,]>,
    Punctuated<syn::WherePredicate, Token![,]>,
) {
    let generic_params = generics.params.clone();
    let where_clause = generics
        .where_clause
        .clone()
        .map(|c| c.predicates)
        .unwrap_or_default();
    (generic_params, where_clause)
}

/// Build [function selector](https://solidity-by-example.org/function-selector/) byte array.
pub fn build_selector<'a>(
    name: &SolIdent,
    params: impl Iterator<Item = &'a syn_solidity::Type>,
) -> [u8; 4] {
    let mut selector = Keccak256::new();
    selector.update(name.to_string());
    selector.update("(");
    for (i, param) in params.enumerate() {
        if i > 0 {
            selector.update(",");
        }
        selector.update(param.to_string());
    }
    selector.update(")");
    let selector = selector.finalize();
    [selector[0], selector[1], selector[2], selector[3]]
}
