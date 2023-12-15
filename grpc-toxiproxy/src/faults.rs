use std::thread;

use grpc_toxiproxy::{DEFAULT_SERVER_PORT, PROXIED_SERVER_PORT};
use toxiproxy_rust::{proxy::ProxyPack, TOXIPROXY};

pub const PROXY_NAME: &str = "grpc-toxiproxy-test";

fn main() {
    setup_proxies();

    // Inject a latency fault
    inject_latency_fault(5000, 0, 1.0);
}

pub fn setup_proxies() {
    thread::spawn(|| {
        // First check if TOXIPROXY is running
        assert!(TOXIPROXY.is_running());

        // Reset all proxies
        let existing_proxies = TOXIPROXY.all().unwrap();
        for proxy in existing_proxies.values() {
            assert!(proxy.delete().is_ok());
        }

        // Populate the proxy

        assert!(TOXIPROXY
            .populate(vec![ProxyPack::new(
                PROXY_NAME.into(),
                format!("127.0.0.1:{}", PROXIED_SERVER_PORT),
                format!("127.0.0.1:{}", DEFAULT_SERVER_PORT),
            )])
            .is_ok());

        // Reset the proxy to make sure no faults
        let proxy_result = TOXIPROXY.find_and_reset_proxy(PROXY_NAME);
        assert!(proxy_result.is_ok());

        let proxy_toxics = proxy_result.as_ref().expect("Failed to get proxy").toxics();
        assert!(proxy_toxics.is_ok());
        assert_eq!(
            0,
            proxy_toxics.as_ref().expect("Failed to get toxics").len()
        );
    })
    .join()
    .expect("Failed to set up proxies");
}

/// Inject latency to the target's response to the source.
///
/// Spawns a thread internally to avoid nested runtimes within the same thread.
pub fn inject_latency_fault(latency_ms: usize, jitter_ms: usize, toxicity: f32) {
    thread::spawn(move || {
        let get_proxy_result = TOXIPROXY.find_proxy(PROXY_NAME);
        assert!(get_proxy_result.is_ok());

        get_proxy_result
            .as_ref()
            .expect("Failed to get proxy")
            .with_latency(
                "upstream".into(),
                latency_ms as u32,
                jitter_ms as u32,
                toxicity,
            );
    })
    .join()
    .expect("Failed to set up latency fault");
}
