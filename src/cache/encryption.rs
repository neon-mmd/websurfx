// Rust guideline compliant 2026-10-09

//! Encryption primitives for securing cached search results.

use chacha20poly1305::ChaCha20Poly1305;
use std::sync::OnceLock;

/// Shared ChaCha20-Poly1305 cipher instance initialized on first cache operation.
pub static CIPHER: OnceLock<ChaCha20Poly1305> = OnceLock::new();
