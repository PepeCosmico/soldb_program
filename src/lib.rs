#![allow(unexpected_cfgs)]

pub mod accounts;
pub mod error;
pub mod instructions;
#[macro_use]
pub mod macros;
pub mod processor;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

use solana_program::declare_id;

declare_id!("SDBPbpwuFzj8zjhf4LjQJwYoy2SAJETeBDGKb8keRpq");
