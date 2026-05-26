//! SHA-256 Hash Verifier
//!
//! Verifies downloaded files against expected SHA-256 hashes.

use std::path::Path;

use sha2::{Digest, Sha256};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// Verifica el hash SHA-256 de un archivo
///
/// # Arguments
/// * `path` - Ruta al archivo a verificar
/// * `expected_hash` - Hash esperado en formato hexadecimal
///
/// # Returns
/// * `Ok(true)` si el hash coincide
/// * `Ok(false)` si el hash no coincide
/// * `Err(e)` si hay un error al leer el archivo o calcular el hash
pub async fn verify_sha256(path: &Path, expected_hash: &str) -> Result<bool, String> {
    let mut file = File::open(path).await.map_err(|e| e.to_string())?;

    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer).await.map_err(|e| e.to_string())?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    let actual_hash = hex_encode(&result);

    // Normalizar hashes: quitar guiones, espacios, convertir a minúsculas
    let expected_clean = normalize_hash(expected_hash);
    let actual_clean = normalize_hash(&actual_hash);

    Ok(expected_clean == actual_clean)
}

/// Convierte bytes a string hexadecimal
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Normaliza un hash SHA-256 quitando caracteres no hexadecimales y convirtiendo a minúsculas
fn normalize_hash(hash: &str) -> String {
    hash.chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect::<String>()
        .to_lowercase()
}

/// Calcula el hash SHA-256 de un archivo y lo retorna como string hexadecimal
/// (útil para debugging o para obtener el hash real de un archivo)
pub async fn calculate_sha256(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).await.map_err(|e| e.to_string())?;

    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer).await.map_err(|e| e.to_string())?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(hex_encode(&result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_hash() {
        assert_eq!(normalize_hash("ABC123"), "abc123");
        assert_eq!(normalize_hash("AB-CD-12-34"), "abcd1234");
        assert_eq!(normalize_hash("ab cd 12 34"), "abcd1234");
    }

    #[test]
    fn test_hex_encode() {
        let bytes = [0x48, 0x65, 0x6c, 0x6c, 0x6f];
        assert_eq!(hex_encode(&bytes), "48656c6c6f");
    }
}