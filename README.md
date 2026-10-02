# wo

`wo.exe` ist eine kleine Windows-Konsolenanwendung (Rust), die anzeigt, wo sich Personen laut Home Assistant gerade befinden.

```
Ralf: Zuhause
Anna: Arbeit
Max:  Unterwegs
```

## Verwendung

| Aufruf | Wirkung |
|---|---|
| `wo` | Aufenthaltsorte anzeigen (beim ersten Start: Einrichtung) |
| `wo /version` | Versionsnummer anzeigen |
| `wo /reset` | Konfiguration löschen und Einrichtung neu starten |
| `wo /?` | Hilfe anzeigen |

Die Parameter funktionieren auch mit `-` oder `--` statt `/` (z. B. `-v`, `--help`), ohne Präfix und unabhängig von der Groß-/Kleinschreibung.

## Einrichtung

Beim ersten Start fragt `wo` nach:

1. **Home-Assistant-URL**, z. B. `http://homeassistant.local:8123`
2. **Long-Lived Access Token** (in Home Assistant unter *Profil → Sicherheit → Langlebige Zugriffstokens* erstellen)

Danach ruft `wo` alle `person.*`- und `device_tracker.*`-Entitäten ab und listet sie nummeriert auf. Du gibst die Nummern der Sensoren an, die angezeigt werden sollen (kommagetrennt, z. B. `1,3`). Enter übernimmt alle.

Wenn eine Person und ihr `device_tracker` beide erscheinen, wähle nur einen davon aus, sonst wird die Person doppelt angezeigt.

## Anzeige

Pro ausgewähltem Sensor wird `<Name>: <Ort>` ausgegeben. Der Abruf erfolgt parallel über `GET /api/states/<entity_id>` mit `Authorization: Bearer <TOKEN>`.

- `home` wird als **Zuhause** angezeigt, `not_home` als **Unterwegs**.
- Andere Zonen werden mit ihrem Namen angezeigt, sonstige Werte unverändert (Rohwert von `state`).

## Konfiguration

Die Konfiguration liegt als JSON in `%APPDATA%\wo\config.json` und wird bei jedem Start automatisch geladen.

> **Hinweis:** Das Token wird im Klartext in dieser Datei gespeichert und bei der Eingabe nicht verborgen.

Neue Personen in Home Assistant erscheinen erst nach `wo /reset`.

## Fehlerbehandlung

- Verbindungs-Timeout 5 s, Gesamt-Timeout 10 s pro Abruf.
- Ungültiges Token (HTTP 401), Netzwerkfehler und fehlende Entitäten erzeugen eine verständliche Fehlermeldung.
- Schlägt nur ein Sensor fehl, werden die übrigen trotzdem angezeigt. Der Exit-Code ist nur dann ungleich 0, wenn gar nichts abgerufen werden konnte.

## Bauen und Release

Voraussetzungen: [Rust](https://rustup.rs) und für Releases die GitHub CLI (`winget install --id GitHub.cli`, danach `gh auth login`).

Das Skript `build.ps1` erledigt beides:

```powershell
.\build.ps1
```
Kompiliert mit `cargo build --release`. Das Ergebnis liegt in `target\release\wo.exe`.

```powershell
.\build.ps1 -Release -Version 0.2.0 -Notes "Beschreibung des Release"
```
Setzt die Version in `Cargo.toml`, baut, committet und pusht die Versionsänderung und erstellt auf GitHub das Release `v0.2.0` mit `wo.exe` als Anhang.

| Parameter | Bedeutung |
|---|---|
| `-Release` | GitHub-Release erstellen |
| `-Version` | Neue Versionsnummer (`x.y.z`), sonst gilt die Version aus `Cargo.toml` |
| `-Notes` | Beschreibung des Release (Standard: `wo <Version>`) |

Jede Versionsnummer kann nur einmal verwendet werden, da das Tag `v<Version>` eindeutig sein muss.
