//! Thin typed client for **KTRL** (Klep2tron Control Server).
//!
//! Blocking HTTP over [`ureq`]; the typed methods cover the endpoints in
//! `.agents/skills/klep2tron-control/SKILL.md`. The `ktrl` binary is a thin
//! `clap` wrapper around this.
//!
//! ```no_run
//! let client = ktrl::Client::from_env();
//! let state = client.state().unwrap();
//! println!("{}", serde_json::to_string_pretty(&state).unwrap());
//! ```

use std::io::Read;
use std::time::Duration;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Http(Box<ureq::Error>),
    Status { code: u16, body: String },
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Http(e) => write!(f, "http error: {e}"),
            Error::Status { code, body } => {
                let body = body.trim();
                if body.is_empty() {
                    write!(f, "server returned {code}")
                } else {
                    write!(f, "server returned {code}: {body}")
                }
            }
            Error::Io(e) => write!(f, "io error: {e}"),
            Error::Json(e) => write!(f, "json error: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<ureq::Error> for Error {
    fn from(e: ureq::Error) -> Self {
        Error::Http(Box::new(e))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

/// Percent-encode the characters that matter for our query strings.
fn encode(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace(' ', "%20")
        .replace('&', "%26")
        .replace('?', "%3F")
        .replace('#', "%23")
}

#[derive(Clone)]
pub struct Client {
    base: String,
    token: Option<String>,
    agent: ureq::Agent,
}

impl Client {
    pub fn new(base: impl Into<String>, token: Option<String>) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(30))
            .build();
        Self {
            base: base.into().trim_end_matches('/').to_string(),
            token: token.filter(|t| !t.is_empty()),
            agent,
        }
    }

    /// `KLEP_CONTROL_URL` (default `http://127.0.0.1:15703`) and
    /// `KLEP_CONTROL_TOKEN`.
    pub fn from_env() -> Self {
        let base = std::env::var("KLEP_CONTROL_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:15703".to_string());
        let token = std::env::var("KLEP_CONTROL_TOKEN").ok();
        Self::new(base, token)
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.base, path.trim_start_matches('/'))
    }

    fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        let mut req = self.agent.get(&self.url(path));
        if let Some(token) = &self.token {
            req = req.set("Authorization", &format!("Bearer {token}"));
        }
        match req.call() {
            Ok(resp) => {
                let mut bytes = Vec::new();
                resp.into_reader().read_to_end(&mut bytes)?;
                Ok(bytes)
            }
            Err(ureq::Error::Status(code, resp)) => Err(Error::Status {
                code,
                body: resp.into_string().unwrap_or_default(),
            }),
            Err(e) => Err(e.into()),
        }
    }

    fn post(&self, path: &str, body: &str) -> Result<String> {
        let mut req = self
            .agent
            .post(&self.url(path))
            .set("Content-Type", "application/json");
        if let Some(token) = &self.token {
            req = req.set("Authorization", &format!("Bearer {token}"));
        }
        match req.send_string(body) {
            Ok(resp) => Ok(resp.into_string()?),
            Err(ureq::Error::Status(code, resp)) => Err(Error::Status {
                code,
                body: resp.into_string().unwrap_or_default(),
            }),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_text(&self, path: &str) -> Result<String> {
        Ok(String::from_utf8_lossy(&self.get_bytes(path)?).into_owned())
    }

    pub fn get_json(&self, path: &str) -> Result<serde_json::Value> {
        Ok(serde_json::from_str(&self.get_text(path)?)?)
    }

    // --- Typed endpoints ----------------------------------------------------

    pub fn version(&self) -> Result<serde_json::Value> {
        self.get_json("/version")
    }

    pub fn state(&self) -> Result<serde_json::Value> {
        self.get_json("/state")
    }

    pub fn scene_tree(&self, depth: u32, root: Option<u32>) -> Result<serde_json::Value> {
        let mut path = format!("/scene_tree?depth={depth}");
        if let Some(root) = root {
            path.push_str(&format!("&root={root}"));
        }
        self.get_json(&path)
    }

    pub fn entity(&self, id: u32) -> Result<serde_json::Value> {
        self.get_json(&format!("/entity/{id}"))
    }

    pub fn mesh(&self, id: u32) -> Result<serde_json::Value> {
        self.get_json(&format!("/mesh/{id}"))
    }

    pub fn material(&self, id: u32) -> Result<serde_json::Value> {
        self.get_json(&format!("/material/{id}"))
    }

    pub fn ui_query(&self, label: Option<&str>) -> Result<serde_json::Value> {
        match label {
            Some(label) => self.get_json(&format!("/ui_query?label={}", encode(label))),
            None => self.get_json("/ui_query"),
        }
    }

    /// PNG bytes of the primary window or of `view` (`camera:<id>` / `rtt:<id>`).
    pub fn screenshot(&self, view: Option<&str>) -> Result<Vec<u8>> {
        match view {
            Some(view) => self.get_bytes(&format!("/screenshot?view={}", encode(view))),
            None => self.get_bytes("/screenshot"),
        }
    }

    /// Fire a KTRL action, merging `extra` into the request body.
    pub fn action(&self, name: &str, extra: Option<serde_json::Value>) -> Result<String> {
        let mut obj = serde_json::Map::new();
        obj.insert("action".into(), serde_json::Value::String(name.to_string()));
        if let Some(serde_json::Value::Object(map)) = extra {
            for (key, value) in map {
                obj.insert(key, value);
            }
        }
        self.post("/action", &serde_json::Value::Object(obj).to_string())
    }

    pub fn set_tile(
        &self,
        x: u32,
        z: u32,
        h: Option<i32>,
        tt: Option<&str>,
    ) -> Result<String> {
        let mut obj = serde_json::Map::new();
        obj.insert("x".into(), x.into());
        obj.insert("z".into(), z.into());
        if let Some(h) = h {
            obj.insert("h".into(), h.into());
        }
        if let Some(tt) = tt {
            obj.insert("tt".into(), tt.into());
        }
        self.action("SetTile", Some(serde_json::Value::Object(obj)))
    }

    pub fn pause(&self, on: bool) -> Result<String> {
        self.post("/pause", &format!("{{\"on\":{on}}}"))
    }

    pub fn step(&self, frames: u32) -> Result<String> {
        self.post("/step", &format!("{{\"frames\":{frames}}}"))
    }

    pub fn key(&self, key: &str, action: &str) -> Result<String> {
        self.post(
            "/key",
            &format!("{{\"key\":\"{}\",\"action\":\"{action}\"}}", encode(key)),
        )
    }

    pub fn ui_click(&self, label: &str, action: &str) -> Result<String> {
        self.post(
            "/ui_click",
            &format!("{{\"label\":\"{}\",\"action\":\"{action}\"}}", encode(label)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_query_values() {
        assert_eq!(encode("Wedge S"), "Wedge%20S");
        assert_eq!(encode("a&b"), "a%26b");
        assert_eq!(encode("camera:42"), "camera:42");
    }

    #[test]
    fn client_trims_trailing_slash() {
        let client = Client::new("http://127.0.0.1:15703/", None);
        assert_eq!(client.base(), "http://127.0.0.1:15703");
    }
}
