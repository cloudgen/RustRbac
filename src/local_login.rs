//! Saved terminal sign-in.
//! requirement-local-login — `local-login` and `local-login-key` under the application home.
//! The password is not stored as plain text. `StatusLog::open` stays in `src/main.rs`.

use std::fs;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

use base64::{engine::general_purpose::STANDARD, Engine};
use getrandom::getrandom;
use sha2::{Digest, Sha256};

use crate::error::AppError;

pub const MAGIC: &str = "RUSTRBACLOCAL1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SignInNote {
    #[default]
    None,
    Saved,
    Cleared,
    Rejected,
}

impl SignInNote {
    pub fn line(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Saved => Some("sign-in saved"),
            Self::Cleared => Some("sign-in cleared"),
            Self::Rejected => Some("sign-in rejected"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SavedBody {
    Absent,
    Ready { user_name: String, password: String },
    Reject,
}

pub struct LocalLogin {
    home: String,
}

impl LocalLogin {
    pub fn new(home: String) -> Self {
        Self { home }
    }

    pub fn save(&self, user_name: &str, password: &str) -> Result<(), AppError> {
        if self.home.is_empty() {
            return Err(AppError::new(
                "The home folder is not set.",
                "export HOME or RUSTRBAC_HOME",
            ));
        }
        if user_name.is_empty() || user_name.as_bytes().contains(&0) {
            return Err(AppError::new(
                "The account name is not valid.",
                "rust-rbac login --user-name NAME",
            ));
        }
        fs::create_dir_all(&self.home).map_err(|_| write_error())?;
        let _ = fs::set_permissions(&self.home, fs::Permissions::from_mode(0o700));
        let key = match self.read_key() {
            Some(key) => key,
            None => {
                let mut key = vec![0u8; 32];
                getrandom(&mut key).map_err(|_| write_error())?;
                self.write_atomic(&self.key_path(), &key)?;
                key
            }
        };
        let body = seal(&key, user_name, password)?;
        self.write_atomic(&self.login_path(), body.as_bytes())
    }

    pub fn load(&self) -> SavedBody {
        if !is_mode_0600(&self.login_path()) {
            return SavedBody::Absent;
        }
        let Some(key) = self.read_key() else {
            return SavedBody::Absent;
        };
        let text = match fs::read_to_string(self.login_path()) {
            Ok(text) => text,
            Err(_) => return SavedBody::Reject,
        };
        match open_body(&key, &text) {
            Some(ready) => ready,
            None => SavedBody::Reject,
        }
    }

    /// Removes `local-login` when it exists. The key file stays.
    pub fn clear(&self) -> Result<bool, AppError> {
        let path = self.login_path();
        if path.is_empty() || !Path::new(&path).exists() {
            return Ok(false);
        }
        fs::remove_file(&path).map_err(|_| {
            AppError::new(
                "The saved sign-in could not be removed.",
                "check RUSTRBAC_HOME",
            )
        })?;
        Ok(true)
    }

    fn login_path(&self) -> String {
        self.named("local-login")
    }

    fn key_path(&self) -> String {
        self.named("local-login-key")
    }

    fn named(&self, name: &str) -> String {
        if self.home.is_empty() {
            return String::new();
        }
        Path::new(&self.home)
            .join(name)
            .to_string_lossy()
            .into_owned()
    }

    fn read_key(&self) -> Option<Vec<u8>> {
        let path = self.key_path();
        if !is_mode_0600(&path) {
            return None;
        }
        let bytes = fs::read(path).ok()?;
        if bytes.len() == 32 {
            Some(bytes)
        } else {
            None
        }
    }

    fn write_atomic(&self, path: &str, bytes: &[u8]) -> Result<(), AppError> {
        let temporary = format!("{path}.tmp");
        {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&temporary)
                .map_err(|_| write_error())?;
            file.write_all(bytes).map_err(|_| write_error())?;
        }
        let _ = fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600));
        fs::rename(&temporary, path).map_err(|_| write_error())?;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
        Ok(())
    }
}

fn write_error() -> AppError {
    AppError::new(
        "The saved sign-in could not be written.",
        "check RUSTRBAC_HOME",
    )
}

fn is_mode_0600(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    let meta = match fs::metadata(path) {
        Ok(meta) if meta.is_file() => meta,
        _ => return false,
    };
    meta.permissions().mode() & 0o777 == 0o600
}

fn seal(key: &[u8], user_name: &str, password: &str) -> Result<String, AppError> {
    let mut nonce = [0u8; 16];
    getrandom(&mut nonce).map_err(|_| write_error())?;
    let mut plain = Vec::new();
    plain.extend_from_slice(user_name.as_bytes());
    plain.push(0);
    plain.extend_from_slice(password.as_bytes());
    let stream = keystream(key, &nonce, plain.len());
    let cipher: Vec<u8> = plain
        .iter()
        .zip(stream.iter())
        .map(|(left, right)| left ^ right)
        .collect();
    let check = hmac_sha256(key, &concat(&nonce, &cipher));
    Ok(format!(
        "{MAGIC}\n{}\n{}\n{}\n",
        STANDARD.encode(nonce),
        STANDARD.encode(check),
        STANDARD.encode(cipher)
    ))
}

fn open_body(key: &[u8], text: &str) -> Option<SavedBody> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() != 4 || lines[0] != MAGIC {
        return None;
    }
    let nonce = STANDARD.decode(lines[1].trim()).ok()?;
    let check = STANDARD.decode(lines[2].trim()).ok()?;
    let cipher = STANDARD.decode(lines[3].trim()).ok()?;
    if nonce.len() != 16 || check.len() != 32 {
        return None;
    }
    let expect = hmac_sha256(key, &concat(&nonce, &cipher));
    if !constant_time_eq(&check, &expect) {
        return None;
    }
    let stream = keystream(key, &nonce, cipher.len());
    let plain: Vec<u8> = cipher
        .iter()
        .zip(stream.iter())
        .map(|(left, right)| left ^ right)
        .collect();
    let split = plain.iter().position(|byte| *byte == 0)?;
    if split == 0 {
        return None;
    }
    let user_name = String::from_utf8(plain[..split].to_vec()).ok()?;
    let password = String::from_utf8(plain[split + 1..].to_vec()).ok()?;
    Some(SavedBody::Ready {
        user_name,
        password,
    })
}

fn concat(nonce: &[u8], cipher: &[u8]) -> Vec<u8> {
    let mut message = Vec::with_capacity(nonce.len() + cipher.len());
    message.extend_from_slice(nonce);
    message.extend_from_slice(cipher);
    message
}

fn keystream(key: &[u8], nonce: &[u8], length: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(length);
    let mut counter: u32 = 0;
    while out.len() < length {
        let mut message = Vec::with_capacity(nonce.len() + 4);
        message.extend_from_slice(nonce);
        message.extend_from_slice(&counter.to_be_bytes());
        out.extend_from_slice(&hmac_sha256(key, &message));
        counter = counter.wrapping_add(1);
    }
    out.truncate(length);
    out
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK: usize = 64;
    let mut key_block = [0u8; BLOCK];
    if key.len() > BLOCK {
        let digest = Sha256::digest(key);
        key_block[..digest.len()].copy_from_slice(&digest);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; BLOCK];
    let mut opad = [0x5cu8; BLOCK];
    for index in 0..BLOCK {
        ipad[index] ^= key_block[index];
        opad[index] ^= key_block[index];
    }
    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(message);
    let inner_digest = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_digest);
    let digest = outer.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
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

#[cfg(test)]
mod tests {
    use super::*;

    fn home() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "rustrbac-local-login-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&path).expect("temp");
        path
    }

    #[test]
    fn round_trip_hides_the_password_and_rejects_a_bad_check() {
        let dir = home();
        let saved = LocalLogin::new(dir.display().to_string());
        let password = "s3cret-value\nline";
        saved.save("admin", password).expect("save");
        let body = fs::read_to_string(dir.join("local-login")).expect("body");
        assert!(body.starts_with("RUSTRBACLOCAL1\n"));
        assert!(!body.contains(password));
        assert_eq!(
            fs::metadata(dir.join("local-login"))
                .expect("meta")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        let key = fs::read(dir.join("local-login-key")).expect("key");
        assert_eq!(key.len(), 32);
        match saved.load() {
            SavedBody::Ready {
                user_name,
                password: opened,
            } => {
                assert_eq!(user_name, "admin");
                assert_eq!(opened, password);
            }
            other => panic!("{other:?}"),
        }
        saved.save("admin", password).expect("reuse");
        assert_eq!(
            fs::read(dir.join("local-login-key")).expect("same key"),
            key
        );

        let mut lines: Vec<String> = body.lines().map(str::to_string).collect();
        lines[2] = "AAAA".to_string();
        fs::write(dir.join("local-login"), format!("{}\n", lines.join("\n"))).expect("tamper");
        let _ = fs::set_permissions(dir.join("local-login"), fs::Permissions::from_mode(0o600));
        assert_eq!(saved.load(), SavedBody::Reject);
        assert!(dir.join("local-login").exists());
        assert!(saved.clear().expect("clear"));
        assert!(!dir.join("local-login").exists());
        assert!(dir.join("local-login-key").exists());
        assert!(!saved.clear().expect("second"));

        saved.save("clerk", "other-secret").expect("again");
        let _ = fs::set_permissions(dir.join("local-login"), fs::Permissions::from_mode(0o644));
        assert_eq!(saved.load(), SavedBody::Absent);
        assert!(dir.join("local-login").exists());

        for note in [SignInNote::Saved, SignInNote::Cleared, SignInNote::Rejected] {
            let line = note.line().expect("line");
            assert!(!line.contains("admin"));
            assert!(!line.contains(password));
        }
        assert_eq!(SignInNote::Saved.line(), Some("sign-in saved"));
        assert_eq!(SignInNote::Cleared.line(), Some("sign-in cleared"));
        assert_eq!(SignInNote::Rejected.line(), Some("sign-in rejected"));
    }
}
