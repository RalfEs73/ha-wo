mod config;
mod ha;
mod secret;

use config::Config;
use std::io::{self, Write};
use std::process::ExitCode;
use std::thread;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    // Präfix "/", "-" oder "--" ist optional und die Groß-/Kleinschreibung egal.
    let norm = arg.as_ref().map(|a| a.trim_start_matches(['/', '-']).to_lowercase());
    let result = match norm.as_deref() {
        Some("version" | "v") => {
            println!("wo {VERSION}");
            Ok(())
        }
        Some("reset" | "r") => config::delete().and_then(|_| setup()),
        Some("?" | "h" | "help") => {
            print_help();
            Ok(())
        }
        None => run(),
        Some(_) => Err(format!(
            "Unbekannter Parameter '{}'. Hilfe: wo ?",
            arg.unwrap_or_default()
        )),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Fehler: {e}");
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    println!(
        "wo {VERSION} - zeigt an, wo sich Personen laut Home Assistant befinden

Aufruf: wo [Parameter]

  (ohne)      Aufenthaltsorte anzeigen (beim ersten Start: Einrichtung)
  /version    Versionsnummer anzeigen
  /reset      Konfiguration löschen und Einrichtung neu starten
  /?          Diese Hilfe anzeigen

Konfiguration: %APPDATA%\\wo\\config.json"
    );
}

fn prompt(text: &str) -> Result<String, String> {
    print!("{text}");
    io::stdout().flush().ok();
    let mut s = String::new();
    match io::stdin().read_line(&mut s) {
        Ok(0) => Err("Eingabe abgebrochen.".into()),
        Ok(_) => Ok(s.trim().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

fn setup() -> Result<(), String> {
    println!("== wo: Ersteinrichtung ==");
    let mut url = prompt("Home-Assistant-URL (z. B. http://homeassistant.local:8123): ")?;
    if !url.contains("://") {
        url = format!("http://{url}");
    }
    let token = secret::read_hidden(|| prompt("Long-Lived Access Token (Eingabe bleibt unsichtbar): "))?;
    if url == "http://" || token.is_empty() {
        return Err("URL und Token dürfen nicht leer sein.".into());
    }

    println!("Verbinde ...");
    let client = ha::Client::new(&url, &token);
    let found = client.list_trackers()?;
    if found.is_empty() {
        return Err("Keine person.*- oder device_tracker.*-Entitäten gefunden.".into());
    }

    println!("\nGefundene Sensoren:");
    for (i, s) in found.iter().enumerate() {
        println!("  {:>2}) {:<35} {} ({})", i + 1, s.entity_id, s.name(), s.location());
    }
    let ans = prompt("\nNummern der Sensoren, die angezeigt werden sollen (kommagetrennt, Enter = alle): ")?;
    let mut selected = Vec::new();
    for part in ans.split(|c: char| c == ',' || c.is_whitespace()).filter(|p| !p.is_empty()) {
        match part.parse::<usize>() {
            Ok(n) if (1..=found.len()).contains(&n) => {
                if !selected.contains(&(n - 1)) {
                    selected.push(n - 1);
                }
            }
            _ => return Err(format!("Ungültige Nummer: '{part}'")),
        }
    }
    if selected.is_empty() {
        selected = (0..found.len()).collect();
    }
    selected.sort_unstable();
    let entities: Vec<String> = selected.iter().map(|&i| found[i].entity_id.clone()).collect();

    let count = entities.len();
    config::save(&Config { base_url: url, token, entities })?;
    println!("Konfiguration gespeichert ({count} Sensoren).\n");
    run()
}

fn run() -> Result<(), String> {
    let Some(cfg) = config::load()? else {
        return setup();
    };
    let client = ha::Client::new(&cfg.base_url, &cfg.token);

    // Parallel abrufen, Reihenfolge bleibt erhalten.
    let results: Vec<Result<ha::State, String>> = thread::scope(|sc| {
        let handles: Vec<_> = cfg.entities.iter().map(|id| sc.spawn(|| client.state(id))).collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap_or_else(|_| Err("Abruf abgestürzt".into())))
            .collect()
    });

    let mut failed = 0;
    let mut rows = Vec::new();
    for (id, r) in cfg.entities.iter().zip(results) {
        match r {
            Ok(s) => rows.push((s.name().to_string(), s.location().to_string())),
            Err(e) => {
                failed += 1;
                eprintln!("{id}: {e}");
            }
        }
    }
    let width = rows.iter().map(|(n, _)| n.chars().count()).max().unwrap_or(0);
    for (name, loc) in &rows {
        println!("{name}:{} {loc}", " ".repeat(width - name.chars().count()));
    }
    if failed == cfg.entities.len() {
        Err("Kein Sensor konnte abgerufen werden.".into())
    } else {
        Ok(())
    }
}
