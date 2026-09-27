//! KTRL enable/bind/port/token configuration.

/// Addresses that keep the server reachable only from the local machine.
fn is_loopback(bind: &str) -> bool {
    let bind = bind.trim();
    if bind.eq_ignore_ascii_case("localhost") {
        return true;
    }
    bind.parse::<std::net::IpAddr>()
        .map(|ip| ip.is_loopback())
        .unwrap_or(false)
}

pub(super) struct ControlConfig {
    pub(super) enabled: bool,
    /// Interface to listen on. Defaults to `127.0.0.1`; set `0.0.0.0` or a LAN
    /// / Tailscale IP to expose the API on the network.
    pub(super) bind: String,
    pub(super) port: u16,
    /// When set, every request must carry `Authorization: Bearer <token>`.
    pub(super) token: Option<String>,
}

impl ControlConfig {
    pub(super) fn load() -> Self {
        let mut enabled = cfg!(debug_assertions);
        let mut bind = "127.0.0.1".to_string();
        let mut port = 15703u16;
        let mut token = std::env::var("KLEP_CONTROL_TOKEN").ok();

        if let Ok(v) = std::env::var("KLEP_CONTROL") {
            enabled = v != "0" && !v.is_empty();
        }
        if let Ok(b) = std::env::var("KLEP_CONTROL_BIND") {
            bind = b;
        }
        if let Ok(p) = std::env::var("KLEP_CONTROL_PORT") {
            if let Ok(p) = p.parse() {
                port = p;
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(content) = std::fs::read_to_string("settings.json") {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(c) = json.get("control") {
                    if let Some(e) = c.get("enabled").and_then(|v| v.as_bool()) {
                        enabled = e;
                    }
                    if let Some(b) = c.get("bind").and_then(|v| v.as_str()) {
                        bind = b.to_string();
                    }
                    if let Some(p) = c.get("port").and_then(|v| v.as_u64()) {
                        port = p as u16;
                    }
                    if let Some(t) = c.get("token").and_then(|v| v.as_str()) {
                        token = Some(t.to_string());
                    }
                }
            }
        }

        Self { enabled, bind, port, token: token.filter(|t| !t.is_empty()) }
    }

    /// `true` when the API is reachable from outside the local machine, which
    /// requires a token.
    pub(super) fn is_exposed(&self) -> bool {
        !is_loopback(&self.bind)
    }
}

#[cfg(test)]
mod tests {
    use super::is_loopback;

    #[test]
    fn loopback_detection() {
        assert!(is_loopback("127.0.0.1"));
        assert!(is_loopback("127.0.0.2"));
        assert!(is_loopback("::1"));
        assert!(is_loopback(" localhost "));
        assert!(!is_loopback("0.0.0.0"));
        assert!(!is_loopback("100.64.0.1"));
        assert!(!is_loopback("192.168.1.10"));
        assert!(!is_loopback("example.com"));
    }
}
