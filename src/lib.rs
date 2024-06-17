//! # kwui
//!
//! A cross-platform GUI library for Rust focused on simplicity and fast development of small tools.
//! Inspired by [sciter](https://sciter.com).
//!
//! [User Guide](https://wanghoi.github.io/kwui)

#![allow(unused, dead_code)]
mod application;
mod script_engine;
mod script_value;

pub use application::*;
pub use script_engine::*;
pub use script_value::*;
