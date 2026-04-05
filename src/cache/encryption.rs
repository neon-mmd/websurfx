use chacha20poly1305::{
    ChaCha20Poly1305,
    aead::generic_array::{
        GenericArray,
        typenum::{UInt, UTerm},
    },
    consts::{B0, B1},
};
use std::sync::OnceLock;

/// Our ChaCha20-Poly1305 cipher instance, lazily initialized.
pub static CIPHER: OnceLock<ChaCha20Poly1305> = OnceLock::new();

/// The type alias for our encryption key, a 32-byte array.
type GenericArrayType = GenericArray<u8, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>;
/// Our encryption key, lazily initialized.
pub static ENCRYPTION_KEY: OnceLock<GenericArrayType> = OnceLock::new();
