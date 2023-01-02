use crate::config::Config;
use sodiumoxide::base64;
use std::sync::{Arc, RwLock};

lazy_static::lazy_static! {
    pub static ref TEMPORARY_PASSWORD:Arc<RwLock<String>> = Arc::new(RwLock::new(Config::get_auto_password(temporary_password_length())));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VerificationMethod {
    OnlyUseTemporaryPassword,
    OnlyUsePermanentPassword,
    UseBothPasswords,
}

// Should only be called in server
pub fn update_temporary_password() {
    *TEMPORARY_PASSWORD.write().unwrap() = Config::get_auto_password(temporary_password_length());
}

// Should only be called in server
pub fn temporary_password() -> String {
    TEMPORARY_PASSWORD.read().unwrap().clone()
}

fn verification_method() -> VerificationMethod {
    let method = Config::get_option("verification-method");
    if method == "use-temporary-password" {
        VerificationMethod::OnlyUseTemporaryPassword
    } else if method == "use-permanent-password" {
        VerificationMethod::OnlyUsePermanentPassword
    } else {
        VerificationMethod::UseBothPasswords // default
    }
}

pub fn temporary_password_length() -> usize {
    let length = Config::get_option("temporary-password-length");
    if length == "8" {
        8
    } else if length == "10" {
        10
    } else {
        6 // default
    }
}

pub fn temporary_enabled() -> bool {
    verification_method() != VerificationMethod::OnlyUsePermanentPassword
}

pub fn permanent_enabled() -> bool {
    verification_method() != VerificationMethod::OnlyUseTemporaryPassword
}

pub fn has_valid_password() -> bool {
    temporary_enabled() && !temporary_password().is_empty()
        || permanent_enabled() && !Config::get_permanent_password().is_empty()
}

const VERSION_LEN: usize = 2;

pub fn encrypt_str_or_original(s: &str, version: &str) -> String {
    if decrypt_str_or_original(s, version).1 {
        log::error!("Duplicate encryption!");
        return s.to_owned();
    }
    if version == "00" {
        if let Ok(s) = encrypt(s.as_bytes()) {
            return version.to_owned() + &s;
        }
    }
    s.to_owned()
}

// String: password
// bool: whether decryption is successful
// bool: whether should store to re-encrypt when load
pub fn decrypt_str_or_original(s: &str, current_version: &str) -> (String, bool, bool) {
    if s.len() > VERSION_LEN {
        let version = &s[..VERSION_LEN];
        if version == "00" {
            if let Ok(v) = decrypt(&s[VERSION_LEN..].as_bytes()) {
                return (
                    String::from_utf8_lossy(&v).to_string(),
                    true,
                    version != current_version,
                );
            }
        }
    }

    (s.to_owned(), false, !s.is_empty())
}

pub fn encrypt_vec_or_original(v: &[u8], version: &str) -> Vec<u8> {
    if decrypt_vec_or_original(v, version).1 {
        log::error!("Duplicate encryption!");
        return v.to_owned();
    }
    if version == "00" {
        if let Ok(s) = encrypt(v) {
            let mut version = version.to_owned().into_bytes();
            version.append(&mut s.into_bytes());
            return version;
        }
    }
    v.to_owned()
}

// Vec<u8>: password
// bool: whether decryption is successful
// bool: whether should store to re-encrypt when load
pub fn decrypt_vec_or_original(v: &[u8], current_version: &str) -> (Vec<u8>, bool, bool) {
    if v.len() > VERSION_LEN {
        let version = String::from_utf8_lossy(&v[..VERSION_LEN]);
        if version == "00" {
            if let Ok(v) = decrypt(&v[VERSION_LEN..]) {
                return (v, true, version != current_version);
            }
        }
    }

    (v.to_owned(), false, !v.is_empty())
}

fn encrypt(v: &[u8]) -> Result<String, ()> {
    if v.len() > 0 {
        symmetric_crypt(v, true).map(|v| base64::encode(v, base64::Variant::Original))
    } else {
        Err(())
    }
}

fn decrypt(v: &[u8]) -> Result<Vec<u8>, ()> {
    if v.len() > 0 {
        base64::decode(v, base64::Variant::Original).and_then(|v| symmetric_crypt(&v, false))
    } else {
        Err(())
    }
}

fn symmetric_crypt(data: &[u8], encrypt: bool) -> Result<Vec<u8>, ()> {
    use sodiumoxide::crypto::secretbox;
    use std::convert::TryInto;

    let mut keybuf = crate::get_uuid();
    keybuf.resize(secretbox::KEYBYTES, 0);
    let key = secretbox::Key(keybuf.try_into().map_err(|_| ())?);
    let nonce = secretbox::Nonce([0; secretbox::NONCEBYTES]);

    if encrypt {
        Ok(secretbox::seal(data, &nonce, &key))
    } else {
        secretbox::open(data, &nonce, &key)
    }
}

mod test {

    #[test]
    fn test() {
        use super::*;

        let version = "00";

        println!("test str");
        let data = "Hello World";
        let encrypted = encrypt_str_or_original(data, version);
        let (decrypted, succ, store) = decrypt_str_or_original(&encrypted, version);
        println!("data: {}", data);
        println!("encrypted: {}", encrypted);
        println!("decrypted: {}", decrypted);
        assert_eq!(data, decrypted);
        assert_eq!(version, &encrypted[..2]);
        assert_eq!(succ, true);
        assert_eq!(store, false);
        let (_, _, store) = decrypt_str_or_original(&encrypted, "99");
        assert_eq!(store, true);
        assert_eq!(decrypt_str_or_original(&decrypted, version).1, false);
        assert_eq!(encrypt_str_or_original(&encrypted, version), encrypted);

        println!("test vec");
        let data: Vec<u8> = vec![1, 2, 3, 4, 5, 6];
        let encrypted = encrypt_vec_or_original(&data, version);
        let (decrypted, succ, store) = decrypt_vec_or_original(&encrypted, version);
        println!("data: {:?}", data);
        println!("encrypted: {:?}", encrypted);
        println!("decrypted: {:?}", decrypted);
        assert_eq!(data, decrypted);
        assert_eq!(version.as_bytes(), &encrypted[..2]);
        assert_eq!(store, false);
        assert_eq!(succ, true);
        let (_, _, store) = decrypt_vec_or_original(&encrypted, "99");
        assert_eq!(store, true);
        assert_eq!(decrypt_vec_or_original(&decrypted, version).1, false);
        assert_eq!(encrypt_vec_or_original(&encrypted, version), encrypted);

        println!("test original");
        let data = version.to_string() + "Hello World";
        let (decrypted, succ, store) = decrypt_str_or_original(&data, version);
        assert_eq!(data, decrypted);
        assert_eq!(store, true);
        assert_eq!(succ, false);
        let verbytes = version.as_bytes();
        let data: Vec<u8> = vec![verbytes[0] as u8, verbytes[1] as u8, 1, 2, 3, 4, 5, 6];
        let (decrypted, succ, store) = decrypt_vec_or_original(&data, version);
        assert_eq!(data, decrypted);
        assert_eq!(store, true);
        assert_eq!(succ, false);
        let (_, succ, store) = decrypt_str_or_original("", version);
        assert_eq!(store, false);
        assert_eq!(succ, false);
        let (_, succ, store) = decrypt_vec_or_original(&vec![], version);
        assert_eq!(store, false);
        assert_eq!(succ, false);
    }
}
