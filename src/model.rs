use rand::distributions::{Alphanumeric, DistString};
use sha2::{Digest, Sha256};
use ulid::Ulid;

pub(crate) mod api;
pub(crate) mod app_setting;
pub(crate) mod url;

pub(crate) fn sha256_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn generate_ulid() -> String {
    Ulid::new().to_string().to_lowercase()
}

pub(crate) fn generate_short() -> String {
    generate_random_sring(6)
}

pub(crate) fn generate_random_sring(len: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), len)
}
