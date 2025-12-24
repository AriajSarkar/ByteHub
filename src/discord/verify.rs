use crabgraph::asym::{Ed25519PublicKey, Ed25519Signature};

pub fn verify_discord_signature(
    public_key_hex: &str,
    timestamp: &str,
    body: &[u8],
    signature_hex: &str,
) -> bool {
    let Ok(public_key) = Ed25519PublicKey::from_hex(public_key_hex) else {
        return false;
    };
    let Ok(signature) = Ed25519Signature::from_hex(signature_hex) else {
        return false;
    };

    let mut message = timestamp.as_bytes().to_vec();
    message.extend_from_slice(body);

    public_key.verify(&message, &signature).is_ok()
}
