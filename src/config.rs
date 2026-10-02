use crate::secret;
use serde::{Deserialize, Serialize};
use std::{env, fs, io, path::PathBuf};

#[derive(Serialize, Deserialize)]
struct Stored {
    base_url: String,
    /// Mit DPAPI verschlüsseltes Token (Hex).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    token_enc: Option<String>,
    /// Nur von Version 0.1.x: Token im Klartext. Wird beim Laden migriert.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    token: Option<String>,
    entities: Vec<String>,
}

pub struct Config {
    pub base_url: String,
    pub token: String,
    /// Entitäten, die angezeigt werden sollen.
    pub entities: Vec<String>,
}

fn path() -> PathBuf {
    let base = env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("wo").join("config.json")
}

pub fn load() -> Result<Option<Config>, String> {
    let p = path();
    let text = match fs::read_to_string(&p) {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("Konfiguration {} nicht lesbar: {e}", p.display())),
    };
    let stored: Stored = serde_json::from_str(&text).map_err(|e| {
        format!("Konfiguration {} ist ungültig ({e}). Bitte 'wo /reset' ausführen.", p.display())
    })?;

    let (token, migrate) = match (&stored.token_enc, &stored.token) {
        (Some(enc), _) => (secret::unprotect(enc)?, false),
        (None, Some(plain)) => (plain.clone(), true),
        (None, None) => return Err("Konfiguration enthält kein Token. Bitte 'wo /reset' ausführen.".into()),
    };
    let cfg = Config { base_url: stored.base_url, token, entities: stored.entities };
    if migrate {
        save(&cfg)?; // Klartext-Token aus älterer Version verschlüsseln
    }
    Ok(Some(cfg))
}

pub fn save(cfg: &Config) -> Result<(), String> {
    let p = path();
    fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
    let stored = Stored {
        base_url: cfg.base_url.clone(),
        token_enc: Some(secret::protect(&cfg.token)?),
        token: None,
        entities: cfg.entities.clone(),
    };
    let json = serde_json::to_string_pretty(&stored).map_err(|e| e.to_string())?;
    fs::write(&p, json).map_err(|e| format!("Konfiguration nicht speicherbar: {e}"))
}

pub fn delete() -> Result<(), String> {
    match fs::remove_file(path()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Konfiguration nicht löschbar: {e}")),
    }
}
