pub mod config;
pub mod protocol;
pub mod handler;
pub mod security;
pub mod websocket;
pub mod tls;
pub mod ssh;
pub mod socks5;
pub mod tcp_fallback;

pub use config::Config;
pub use handler::start_proxy;
pub use protocol::ProtocolDetector;
pub use security::SecurityManager;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_STATUS: &str = "@BSPROXY";
