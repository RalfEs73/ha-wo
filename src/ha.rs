use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize, Clone)]
pub struct State {
    pub entity_id: String,
    pub state: String,
    #[serde(default)]
    pub attributes: Attributes,
}

#[derive(Deserialize, Default, Clone)]
pub struct Attributes {
    pub friendly_name: Option<String>,
}

impl State {
    pub fn name(&self) -> &str {
        self.attributes.friendly_name.as_deref().unwrap_or(&self.entity_id)
    }

    /// Übersetzt die Standard-Zustände; Zonennamen bleiben unverändert.
    pub fn location(&self) -> &str {
        match self.state.as_str() {
            "home" => "Zuhause",
            "not_home" => "Unterwegs",
            other => other,
        }
    }
}

pub struct Client {
    base: String,
    auth: String,
    agent: ureq::Agent,
}

impl Client {
    pub fn new(base: &str, token: &str) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .build();
        Client {
            base: base.trim().trim_end_matches('/').to_string(),
            auth: format!("Bearer {}", token.trim()),
            agent,
        }
    }

    fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, String> {
        let url = format!("{}{}", self.base, path);
        match self.agent.get(&url).set("Authorization", &self.auth).call() {
            Ok(r) => r.into_json().map_err(|e| format!("Ungültige Antwort von {url}: {e}")),
            Err(ureq::Error::Status(401, _)) => Err("Token abgelehnt (401). Bitte 'wo reset' ausführen.".into()),
            Err(ureq::Error::Status(404, _)) => Err(format!("Nicht gefunden (404): {path}")),
            Err(ureq::Error::Status(c, _)) => Err(format!("Home Assistant antwortete mit HTTP {c} ({path})")),
            Err(ureq::Error::Transport(t)) => Err(format!("Verbindung zu {} fehlgeschlagen: {t}", self.base)),
        }
    }

    /// Alle person.*- und device_tracker.*-Entitäten (nur für das Setup).
    pub fn list_trackers(&self) -> Result<Vec<State>, String> {
        let mut v: Vec<State> = self
            .get::<Vec<State>>("/api/states")?
            .into_iter()
            .filter(|s| s.entity_id.starts_with("person.") || s.entity_id.starts_with("device_tracker."))
            .collect();
        v.sort_by(|a, b| a.entity_id.cmp(&b.entity_id));
        Ok(v)
    }

    pub fn state(&self, entity_id: &str) -> Result<State, String> {
        self.get(&format!("/api/states/{entity_id}"))
    }
}
