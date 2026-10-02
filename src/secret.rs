//! Verschlüsselung über die Windows Data Protection API (DPAPI).
//! Die Daten lassen sich nur vom selben Windows-Benutzer auf demselben Rechner entschlüsseln.

use std::ptr;
use std::slice;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB,
};
use windows_sys::Win32::System::Console::{
    GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_ECHO_INPUT, STD_INPUT_HANDLE,
};

/// Verschlüsselt `plain` und liefert das Ergebnis als Hex-String.
pub fn protect(plain: &str) -> Result<String, String> {
    let input = CRYPT_INTEGER_BLOB { cbData: plain.len() as u32, pbData: plain.as_ptr() as *mut u8 };
    let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: ptr::null_mut() };
    // SAFETY: `input` zeigt auf gültigen Speicher; `out` wird von Windows befüllt und unten freigegeben.
    let ok = unsafe {
        CryptProtectData(&input, ptr::null(), ptr::null(), ptr::null(), ptr::null(), 0, &mut out)
    };
    if ok == 0 {
        return Err("Token konnte nicht verschlüsselt werden.".into());
    }
    Ok(take_blob(out).iter().map(|b| format!("{b:02x}")).collect())
}

/// Entschlüsselt einen mit [`protect`] erzeugten Hex-String.
pub fn unprotect(hex: &str) -> Result<String, String> {
    const MSG: &str = "Token konnte nicht entschlüsselt werden (anderer Benutzer oder Rechner?). Bitte 'wo /reset' ausführen.";
    if hex.len() % 2 != 0 {
        return Err(MSG.into());
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        bytes.push(hex.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok()).ok_or(MSG)?);
    }
    let input = CRYPT_INTEGER_BLOB { cbData: bytes.len() as u32, pbData: bytes.as_mut_ptr() };
    let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: ptr::null_mut() };
    // SAFETY: wie in `protect`.
    let ok = unsafe {
        CryptUnprotectData(&input, ptr::null_mut(), ptr::null(), ptr::null(), ptr::null(), 0, &mut out)
    };
    if ok == 0 {
        return Err(MSG.into());
    }
    String::from_utf8(take_blob(out)).map_err(|_| MSG.into())
}

/// Kopiert die von Windows allokierten Daten und gibt den Originalpuffer frei.
fn take_blob(blob: CRYPT_INTEGER_BLOB) -> Vec<u8> {
    // SAFETY: Windows liefert `cbData` gültige Bytes ab `pbData`, freizugeben mit LocalFree.
    unsafe {
        let v = slice::from_raw_parts(blob.pbData, blob.cbData as usize).to_vec();
        LocalFree(blob.pbData as _);
        v
    }
}

/// Liest eine Zeile von der Konsole, ohne die Eingabe anzuzeigen
/// (fällt auf normale Eingabe zurück, wenn stdin keine Konsole ist).
pub fn read_hidden(read: impl FnOnce() -> Result<String, String>) -> Result<String, String> {
    // SAFETY: reine Konsolen-API-Aufrufe mit gültigem Handle und lokalem Modus-Wert.
    unsafe {
        let h = GetStdHandle(STD_INPUT_HANDLE);
        let mut mode = 0;
        if GetConsoleMode(h, &mut mode) == 0 {
            return read();
        }
        SetConsoleMode(h, mode & !ENABLE_ECHO_INPUT);
        let r = read();
        SetConsoleMode(h, mode);
        println!();
        r
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn roundtrip() {
        let enc = super::protect("geheim-äöü").unwrap();
        assert!(!enc.contains("geheim"));
        assert_eq!(super::unprotect(&enc).unwrap(), "geheim-äöü");
        assert!(super::unprotect("zz").is_err());
    }
}
