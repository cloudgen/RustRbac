//! PBKDF2-HMAC-SHA256 passwords.
//! requirement-rbac — stored passwords are pbkdf2_sha256, never plaintext.

use base64::{engine::general_purpose::STANDARD, Engine};
use getrandom::getrandom;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

pub struct PasswordHash;

impl PasswordHash {
    pub const ITERATIONS: u32 = 260_000;
    pub const PREFIX: &'static str = "pbkdf2_sha256";

    pub fn hash(&self, password: &str) -> String {
        let mut salt = [0u8; 16];
        getrandom(&mut salt).expect("salt");
        self.hash_with_salt(password, &salt, Self::ITERATIONS)
    }

    fn hash_with_salt(&self, password: &str, salt: &[u8], rounds: u32) -> String {
        let mut digest = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, rounds, &mut digest);
        format!(
            "{}${}${}${}",
            Self::PREFIX,
            rounds,
            STANDARD.encode(salt),
            STANDARD.encode(digest)
        )
    }

    /// True only when the stored value is this scheme and the password matches.
    /// The compare walks every digest byte.
    pub fn verify(&self, password: &str, stored: &str) -> bool {
        let parts: Vec<&str> = stored.split('$').collect();
        if parts.len() != 4 || parts[0] != Self::PREFIX {
            return false;
        }
        let rounds: u32 = match parts[1].parse() {
            Ok(value) => value,
            Err(_) => return false,
        };
        let salt = match STANDARD.decode(parts[2]) {
            Ok(value) => value,
            Err(_) => return false,
        };
        let expected = match STANDARD.decode(parts[3]) {
            Ok(value) => value,
            Err(_) => return false,
        };
        let mut got = vec![0u8; expected.len()];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, rounds, &mut got);
        constant_time_eq(&got, &expected)
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0u8;
    for (one, two) in left.iter().zip(right.iter()) {
        diff |= one ^ two;
    }
    diff == 0
}
