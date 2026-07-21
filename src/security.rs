use std::sync::Arc;
use dashmap::DashMap;
use log::{info, warn};
use std::net::IpAddr;
use tokio::time::{interval, Duration};

#[derive(Clone)]
pub struct SecurityManager {
    connections: Arc<DashMap<IpAddr, usize>>,
    pub max_per_ip: usize,
}

impl SecurityManager {
    pub fn new() -> Self {
        let manager = SecurityManager {
            connections: Arc::new(DashMap::new()),
            max_per_ip: 15,
        };

        // Limpeza periódica
        let conns = manager.connections.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(60));
            loop {
                ticker.tick().await;
                conns.retain(|_, count| *count > 0);
            }
        });

        manager
    }

    pub fn allow_connection(&self, ip: IpAddr) -> bool {
        let entry = self.connections.entry(ip).or_insert(0);
        if *entry < self.max_per_ip {
            *entry += 1;
            true
        } else {
            warn!("🚫 IP bloqueado por excesso de conexões: {}", ip);
            false
        }
    }

    pub fn release(&self, ip: IpAddr) {
        if let Some(mut count) = self.connections.get_mut(&ip) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }
}
