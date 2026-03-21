use crate::firmware::Firmware;
use crate::Error;
use sha2::{Digest, Sha512};

pub const ENV_VAR: &str = "MERGE_TOOL_ED25519_PRIVATE_KEY";
pub const ENV_FILE_VAR: &str = "MERGE_TOOL_ED25519_PRIVATE_KEY_FILE";

fn decode_private_key_hex(input: &str) -> Result<[u8; 32], Error> {
    use std::convert::TryInto;
    let bytes = hex::decode(input.trim()).map_err(|_| Error::InvalidPrivateKey)?;
    bytes.try_into().map_err(|_| Error::InvalidPrivateKey)
}

fn image_digest(fw: &Firmware) -> [u8; 64] {
    let image_len = fw.image_length();
    let mut sha = Sha512::new();
    Digest::input(&mut sha, &fw.data[64..image_len]);

    let mut digest = [0u8; 64];
    digest.copy_from_slice(&sha.result());
    digest
}

/// Generate a new random Ed25519 private key, returned as raw 32-byte seed.
pub fn generate_private_key() -> [u8; 32] {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    SigningKey::generate(&mut OsRng).to_bytes()
}

/// Derive the 32-byte public key from a private key seed.
pub fn public_key_bytes(private_key: &[u8; 32]) -> [u8; 32] {
    use ed25519_dalek::SigningKey;
    SigningKey::from_bytes(private_key)
        .verifying_key()
        .to_bytes()
}

/// Load the private key seed from environment variables.
///
/// If `MERGE_TOOL_ED25519_PRIVATE_KEY_FILE` is set, the file content is read and used
/// as a hex-encoded 32-byte private key seed.
/// Otherwise `MERGE_TOOL_ED25519_PRIVATE_KEY` is used directly as a hex-encoded seed.
pub fn load_private_key_from_env() -> Result<Option<[u8; 32]>, Error> {
    match std::env::var(ENV_FILE_VAR) {
        Ok(path) => {
            let content = std::fs::read_to_string(path).map_err(|_| Error::InvalidPrivateKey)?;
            return decode_private_key_hex(&content).map(Some);
        }
        Err(std::env::VarError::NotPresent) => {}
        Err(_) => return Err(Error::InvalidPrivateKey),
    }

    match std::env::var(ENV_VAR) {
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(_) => Err(Error::InvalidPrivateKey),
        Ok(val) => decode_private_key_hex(&val).map(Some),
    }
}

/// Verify a firmware image signature against the given public key.
///
/// The 64-byte signature is expected at the first 64 bytes of the image.
/// It must cover the SHA-512 digest of everything from byte 64 onwards.
pub fn verify(fw: &Firmware, public_key: &[u8; 32]) -> Result<(), Error> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    use std::convert::TryInto;

    let verifying_key =
        VerifyingKey::from_bytes(public_key).map_err(|_| Error::InvalidSignature)?;
    let sig_bytes: [u8; 64] = fw
        .data
        .get(..64)
        .and_then(|s| s.try_into().ok())
        .ok_or(Error::InvalidSignature)?;
    let signature = Signature::from_bytes(&sig_bytes);
    let digest = image_digest(fw);
    verifying_key
        .verify(&digest, &signature)
        .map_err(|_| Error::InvalidSignature)
}

/// Sign a firmware image with the given private key seed.
///
/// The 64-byte signature is written to the first 64 bytes of the image and
/// covers the SHA-512 digest of everything from byte 64 onwards
/// (CRC + header + code).
/// The key-id and CRC must already be written into the image before calling this
/// (i.e. `configure_header` must have run first).
pub fn sign(fw: &mut Firmware, private_key: &[u8; 32]) -> Result<(), Error> {
    use ed25519_dalek::{Signer, SigningKey};
    let signing_key = SigningKey::from_bytes(private_key);
    let digest = image_digest(fw);
    let signature = signing_key.sign(&digest);
    fw.data[..64].copy_from_slice(&signature.to_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_key_hex(byte: u8) -> String {
        hex::encode([byte; 32])
    }

    fn test_key(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    fn cleanup_env() {
        std::env::remove_var(ENV_FILE_VAR);
        std::env::remove_var(ENV_VAR);
    }

    #[test]
    #[serial]
    fn load_private_key_from_file() {
        cleanup_env();

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("merge-tool-key-{}.txt", unique));
        std::fs::write(&path, format!("{}\n", test_key_hex(0x11))).unwrap();

        std::env::set_var(ENV_FILE_VAR, path.to_string_lossy().to_string());
        std::env::set_var(ENV_VAR, test_key_hex(0x22));

        let key = load_private_key_from_env().unwrap().unwrap();
        assert_eq!(key, test_key(0x11));

        cleanup_env();
        let _ = std::fs::remove_file(path);
    }

    #[test]
    #[serial]
    fn file_env_var_takes_precedence_over_inline_key() {
        cleanup_env();

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("merge-tool-key-{}.txt", unique));
        std::fs::write(&path, test_key_hex(0x33)).unwrap();

        std::env::set_var(ENV_VAR, test_key_hex(0x44));
        std::env::set_var(ENV_FILE_VAR, path.to_string_lossy().to_string());

        let key = load_private_key_from_env().unwrap().unwrap();
        assert_eq!(key, test_key(0x33));

        cleanup_env();
        let _ = std::fs::remove_file(path);
    }
}
