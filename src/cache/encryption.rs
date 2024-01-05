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

pub static CIPHER: OnceLock<
    ChaChaPoly1305<
        StreamCipherCoreWrapper<ChaChaCore<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>>>,
    >,
> = OnceLock::new();

pub static ENCRYPTION_KEY: OnceLock<
    GenericArray<u8, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>,
> = OnceLock::new();
