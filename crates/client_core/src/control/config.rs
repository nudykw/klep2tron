//! KTRL enable/port configuration.

pub(super) struct ControlConfig {
    pub(super) enabled: bool,
    pub(super) port: u16,
    /// When set, every request must carry `Authorization: Bearer <token>`.
    pub(super) token: Option<String>,
}

impl ControlConfig {
    pub(super) fn load() -> Self {
        let mut enabled = cfg!(debug_assertions);
        let mut port = 15703u16;
        let mut token = std::env::var("KLEP_CONTROL_TOKEN").ok();

        if let Ok(v) = std::env::var("KLEP_CONTROL") {
            enabled = v != "0" && !v.is_empty();
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
                    if let Some(p) = c.get("port").and_then(|v| v.as_u64()) {
                        port = p as u16;
                    }
                    if let Some(t) = c.get("token").and_then(|v| v.as_str()) {
                        token = Some(t.to_string());
                    }
                }
            }
        }

        Self { enabled, port, token: token.filter(|t| !t.is_empty()) }
    }
}
