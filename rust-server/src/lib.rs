//! BrainStorm library — exports public modules for integration tests.
//!
//! This crate is both a binary (main.rs) and a library (lib.rs).
//! The library exports domain types and services for testing.

pub mod config;
pub mod db;
pub mod domain;
pub mod error;
pub mod middleware;
pub mod security;
pub mod services;
