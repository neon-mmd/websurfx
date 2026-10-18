use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use std::sync::OnceLock;

/// Our ChaCha20-Poly1305 cipher instance, lazily initialized.
pub static CIPHER: OnceLock<ChaCha20Poly1305> = OnceLock::new();

/// The type alias for our encryption key, a 12-byte array.
type GenericArrayType = Nonce;
/// Our encryption key, lazily initialized.
pub static ENCRYPTION_KEY: OnceLock<GenericArrayType> = OnceLock::new();
