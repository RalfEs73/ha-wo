<#
.SYNOPSIS
    Baut wo.exe und erstellt optional ein GitHub-Release.

.EXAMPLE
    .\build.ps1
    Nur kompilieren.

.EXAMPLE
    .\build.ps1 -Release -Version 0.2.0 -Notes "Neue Hilfe und Sensorauswahl"
    Version in Cargo.toml setzen, bauen, committen, pushen und Release v0.2.0 erstellen.
#>
[CmdletBinding()]
param(
    [switch]$Release,
    [string]$Version,
    [string]$Notes
)

$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

function Invoke-Native {
    param([string]$Cmd, [string[]]$Arguments)
    & $Cmd @Arguments
    if ($LASTEXITCODE -ne 0) { throw "'$Cmd $($Arguments -join ' ')' ist fehlgeschlagen (Exit-Code $LASTEXITCODE)." }
}

$cargoToml = Join-Path $PSScriptRoot 'Cargo.toml'

# Neue Versionsnummer in Cargo.toml eintragen (nur die erste version = "..." im [package]-Block).
if ($Version) {
    if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "Ungültige Version '$Version'. Erwartet: x.y.z" }
    $text = Get-Content $cargoToml -Raw
    $text = [regex]::Replace($text, '(?m)^version\s*=\s*"[^"]*"', "version = `"$Version`"", 1)
    Set-Content $cargoToml $text -NoNewline -Encoding utf8
}

$current = ([regex]::Match((Get-Content $cargoToml -Raw), '(?m)^version\s*=\s*"([^"]*)"')).Groups[1].Value
Write-Host "Baue wo $current ..." -ForegroundColor Cyan
Invoke-Native cargo @('build', '--release')

$exe = Join-Path $PSScriptRoot 'target\release\wo.exe'
if (-not (Test-Path $exe)) { throw "$exe wurde nicht erzeugt." }
Write-Host "Fertig: $exe" -ForegroundColor Green

if (-not $Release) { return }

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    throw "GitHub CLI 'gh' nicht gefunden. Installation: winget install --id GitHub.cli"
}
if (-not $Notes) { $Notes = "wo $current" }
$tag = "v$current"

# Versionsänderung committen und pushen, damit das Tag auf dem richtigen Stand liegt.
if (git status --porcelain Cargo.toml Cargo.lock) {
    Invoke-Native git @('add', 'Cargo.toml', 'Cargo.lock')
    Invoke-Native git @('commit', '-m', "Version $current")
}
Invoke-Native git @('push')

Write-Host "Erstelle Release $tag ..." -ForegroundColor Cyan
Invoke-Native gh @('release', 'create', $tag, $exe, '--title', "wo $current", '--notes', $Notes)
