//! Crate handling Erlang syntax and typing
#![feature(iter_advance_by)]
#![warn(missing_docs)]
extern crate colored;
extern crate derivative;
extern crate lazy_static;
extern crate libironclad_util;
extern crate nom;
extern crate num;
extern crate num_bigint;

pub mod erl_syntax;
pub mod error;
pub mod exit_codes;
pub mod literal;
pub mod project;
pub mod record_def;
pub mod source_loc;
pub mod typing;
