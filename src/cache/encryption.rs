use chacha20poly1305::{
    consts::{B0, B1},
    ChaChaPoly1305,
};
use std::sync::OnceLock;

use chacha20::{
    cipher::{
        generic_array::GenericArray,
        typenum::{UInt, UTerm},
        StreamCipherCoreWrapper,
    },
    ChaChaCore,
};

/// The ChaCha20 core wrapped in a stream cipher for use in ChaCha20-Poly1305 authenticated encryption.
type StreamCipherCoreWrapperType =
    StreamCipherCoreWrapper<ChaChaCore<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>>>;
/// Our ChaCha20-Poly1305 cipher instance, lazily initialized.
pub static CIPHER: OnceLock<ChaChaPoly1305<StreamCipherCoreWrapperType>> = OnceLock::new();

/// The type alias for our encryption key, a 32-byte array.
type GenericArrayType = GenericArray<u8, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>;
/// Our encryption key, lazily initialized.
pub static ENCRYPTION_KEY: OnceLock<GenericArrayType> = OnceLock::new();
