//! The Baja library
//!
//! Code for sending and receiving data over a TLS-encrypted link, inside the
//! Veracruz runtime.
//!
//! ## Authors
//!
//! The Veracruz Development Team.
//!
//! ## Licensing and copyright notice
//!
//! See the `LICENSE.markdown` file in the Veracruz root directory for
//! information on licensing and copyright.

#![cfg_attr(feature = "sgx", no_std)]
#![feature(rustc_private)]

#[cfg(feature = "sgx")]
#[macro_use]
extern crate sgx_tstd as std;

pub mod baja;
pub use self::baja::*;
pub mod baja_session;
pub use self::baja_session::*;
pub mod error;
pub use self::error::BajaError;
