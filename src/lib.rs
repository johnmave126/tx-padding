//! Padding and unpadding of messages with random prepended bytes and trailing zeros
//!
//! This crate provides a padding scheme compatible with `block-padding` crate
//! in which random bytes are prepended and zeros are appended.
//!
//! For a message of length `size`, a buffer of length
//! `block_size * ((size + 1) / block_size) + 2 * block_size` is required for padding. Let `pad_len` be
//! `(-size - 2) % block_size + 2`. Apparently `pad_len` is the number of bytes to pad the message into
//! multiple of `block_size`. The padding scheme appends `pad_len + 1` bytes at the front of the
//! message, where the lower `log(block_size)` bits of the first byte stores `pad_len - 2` and the rest
//! of the bits in the padding are random. At this point the message needs `block_size - 1` more
//! bytes to form multiple of `block_size` and we will just pad `\0` at the end.
//!
//! So `TxPadding<N>` comes with a type parameter `N` which specify the block size to use which is
//! essential for unpadding. `N` must be a power of 2.
//!
//! ```
//! use tx_padding::{TxPadding, Padding};
//! use tx_padding::consts::{U8};
//!
//! let msg = b"test";
//! let n = msg.len();
//! let mut buffer = [0xff; 16];
//! buffer[..n].copy_from_slice(msg);
//! let padded_msg = TxPadding::<U8>::pad(&mut buffer, n, 8).unwrap();
//! assert_eq!(&padded_msg[5..], b"test\x00\x00\x00\x00\x00\x00\x00");
//! assert_eq!((padded_msg[0] & 0x7) + 2, 4);
//! assert_eq!(TxPadding::<U8>::unpad(&padded_msg).unwrap(), msg);
//! ```
//! ```
//! use tx_padding::{TxPadding, Padding};
//! use tx_padding::consts::{U8};
//! let mut buffer = [0xff; 8];
//! assert!(TxPadding::<U8>::pad(&mut buffer, 5, 8).is_err());
//! ```
//!
//! `pad_block` will always return `PadError` since it is not intended to be called. `pad` will
//! return `PadError` if `block_size > 511`, `block_size` mismatch type parameter `N` or buffer
//! is not sufficiently large, which is stricter than the requirement of the `Padding` trait.
#![no_std]

pub use block_padding::{PadError, Padding, UnpadError};

use core::convert::Infallible;
use core::marker::PhantomData;

use consts::{U1, U256};
pub use typenum::consts;

use typenum::marker_traits::{NonZero, PowerOfTwo, Unsigned};
use typenum::operator_aliases::{Gr, LeEq};
use typenum::type_operators::{IsGreater, IsLessOrEqual};

use rand::RngCore;

#[cfg(not(features = "thread_rng"))]
type DefaultRng = rand::rngs::OsRng;
#[cfg(features = "thread_rng")]
type DefaultRng = rand::ThreadRng;

pub enum TxPadding<N> {
    _Phantom(Infallible, PhantomData<N>),
}

impl<N> Padding for TxPadding<N>
where
    N: PowerOfTwo + Unsigned + IsLessOrEqual<U256> + IsGreater<U1>,
    LeEq<N, U256>: NonZero,
    Gr<N, U1>: NonZero,
{
    fn pad_block(_block: &mut [u8], _pos: usize) -> Result<(), PadError> {
        Err(PadError)
    }

    fn unpad(data: &[u8]) -> Result<&[u8], UnpadError> {
        if data.is_empty() {
            Err(UnpadError)?
        }
        let l = data.len();
        let block_size = N::to_usize();
        let pad_zero = block_size - 1;
        let pad_len = (data[0] & (pad_zero as u8)) as usize + 2;
        if l < pad_len + block_size {
            Err(UnpadError)?
        }
        if data[l - pad_zero..l].iter().any(|&v| v != 0) {
            Err(UnpadError)?
        }

        Ok(&data[1 + pad_len..l - pad_zero])
    }

    fn pad(buf: &mut [u8], pos: usize, block_size: usize) -> Result<&mut [u8], PadError> {
        if block_size != N::to_usize() {
            Err(PadError)?
        }
        let block_size = N::to_usize();
        let be = block_size * ((pos + 1) / block_size + 2);
        if buf.len() < be {
            Err(PadError)?
        }

        let pad_zero = block_size - 1;
        let pad_len = ((-(pos as isize) - 2).rem_euclid(block_size as isize)) as usize + 2;
        buf.copy_within(..pos, 1 + pad_len);
        if DefaultRng::default()
            .try_fill_bytes(&mut buf[1..1 + pad_len])
            .is_err()
        {
            Err(PadError)?
        }
        buf[0] = !((block_size - 1) as u8) | (pad_len - 2) as u8;

        // SAFETY: will use slice::fill after it stabilizes
        unsafe {
            core::ptr::write_bytes(buf[be - pad_zero..be].as_mut_ptr(), 0, pad_zero);
        }

        Ok(&mut buf[..be])
    }
}
