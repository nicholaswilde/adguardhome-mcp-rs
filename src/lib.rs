pub mod adguard;
pub mod config;
pub mod error;
pub mod mcp;
pub mod server;
pub mod tools;

#[cfg(test)]
pub mod test_utils {
    use std::sync::Mutex;
    pub static ENV_LOCK: Mutex<()> = Mutex::new(());
}
