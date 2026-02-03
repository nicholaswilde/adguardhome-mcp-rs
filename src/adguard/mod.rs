pub mod client;
pub mod models;

pub use client::AdGuardClient;
pub use models::*;

#[cfg(test)]
mod tests;
