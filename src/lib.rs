mod config;
mod protocol;
mod security;
mod websocket;
mod tls;
mod ssh;
mod socks5;
mod tcp_fallback;

pub use config::Config;
pub use protocol::ProtocolDetector;
pub use security::SecurityManager;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_STATUS: &str = "@BSPROXY";
