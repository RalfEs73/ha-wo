use serde::{Deserialize, Serialize};
use std::{env, fs, io, path::PathBuf};

#[derive(Serialize, Deserialize)]
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
    match fs::read_to_string(&p) {
        Ok(s) => serde_json::from_str(&s).map(Some).map_err(|e| {
            format!("Konfiguration {} ist ungültig ({e}). Bitte 'wo reset' ausführen.", p.display())
        }),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Konfiguration {} nicht lesbar: {e}", p.display())),
    }
}

pub fn save(cfg: &Config) -> Result<(), String> {
    let p = path();
    fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(&p, json).map_err(|e| format!("Konfiguration nicht speicherbar: {e}"))
}

pub fn delete() -> Result<(), String> {
    match fs::remove_file(path()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Konfiguration nicht löschbar: {e}")),
    }
}
