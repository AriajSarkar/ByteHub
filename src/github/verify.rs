use crabgraph::mac::hmac_sha256;

pub fn verify_github_signature(secret: &str, payload: &[u8], signature: &str) -> bool {
    let Some(hex_sig) = signature.strip_prefix("sha256=") else {
        return false;
    };
    let Ok(expected) = hex::decode(hex_sig) else {
        return false;
    };
    let Ok(computed) = hmac_sha256(secret.as_bytes(), payload) else {
        return false;
    };
    constant_time_eq(&computed, &expected)
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}
