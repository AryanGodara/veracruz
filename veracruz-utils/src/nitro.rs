//! Structs needed for AWS Nitro Enclaves, both inside and outside of the 
//! enclave
//!
//! ## Authors
//!
//! The Veracruz Development Team.
//!
//! ## Licensing and copyright notice
//!
//! See the `LICENSE.markdown` file in the Veracruz root directory for
//! information on licensing and copyright.

use serde::{Deserialize, Serialize};

use byteorder::{ByteOrder, LittleEndian};
use err_derive::Error;
use nix::errno::Errno::EINTR;
use nix::sys::socket::{recv, send, MsgFlags};
use std::os::unix::io::RawFd;

/// The Status value returned by the Nitro enclave for operations
/// This is intended to be received as a bincode serialized 
/// `NitroRootEnclaveMessage::Status`
#[derive(Serialize, Deserialize, Debug)]
pub enum NitroStatus {
    /// The operation generating the message succeeded
    Success,
    /// The operation generating the message failed
    Fail,
    /// The requested operation is not yet implemented
    Unimplemented,
}

/// An enumerated type describing messages passed between the Runtime Manager
/// and the Nitro Root Enclave
/// These messages are inteded to be serialized using bincode before transport,
/// and deserialized using bincode after transport
#[derive(Serialize, Deserialize, Debug)]
pub enum NitroRootEnclaveMessage {
    /// A message generated by an operation that did not return data, but did
    /// return a status.
    /// Most operations return data, but if they fail, they will return a
    /// status set to `NitroStatus::Fail` (or `NitroStatus::Unimplemented` if
    /// it is not implmeneted).
    /// Parameters:
    /// NitroStatus - the Status
    Status(NitroStatus),
    /// A request to fetch the firmware version from the Nitro Root Enclave
    FetchFirmwareVersion,
    /// A response to the `FetchFirmwareVersion` message, it contains the
    /// firmware version of the Nitro Root Enclave, as a string
    FirmwareVersion(String),
    /// A request to set the Runtime Manager hash that the Nitro Root Enclave
    /// expects during the proxy attestation. This is a dirty hack to allow
    /// TrustZone to work (as we are faking attestation)
    /// The Vec<u8> input is the hash value
    SetRuntimeManagerHashHack(Vec<u8>),
    /// A request to start the native attestation process.
    /// This is usually initiated from the Proxy Attestation Service
    /// The values:
    /// Vec<u8> - The 128-bit challenge value generated by the caller
    /// i32     - A device ID set by the caller. Will be used by the enclave
    ///           in future operations
    NativeAttestation(Vec<u8>, i32),
    /// A response to the NativeAttestation message. This is generated by the
    /// enclave.
    /// The parameters:
    /// Vec<u8> - The native attestation token generated by the enclave
    /// Vec<u8> - The public key, randomly generated by the enclave, to be used
    ///           as the device-specific proxy public key
    TokenData(Vec<u8>, Vec<u8>),
    /// A request (usually initiated by the Runtime Manager enclave) to start the
    /// proxy attestation process.
    /// The parameters:
    /// Vec<u8> - The 128-bit challenge value generated by the caller
    /// Vec<u8> - The native token value, generated by the caller. This is also,
    ///           referred to as the local attestation token on some platforms
    /// String  - The (supposedly) randomly generated enclave name
    ProxyAttestation(Vec<u8>, Vec<u8>, String),
    /// A response to the ProxyAttestation message. This is the PSA token
    /// information that will be sent to the Proxy Attestation Service
    /// The parameters:
    /// Vec<u8> - The PSA Attestation token to be forwarded to the Proxy
    ///           Attestation Service
    /// Vec<u8> - The public key
    /// u32     - The device ID that identifies the Nitro Root Enclave to the
    ///           Proxy Attestation Service
    PSAToken(Vec<u8>, Vec<u8>, u32),
}

/// An enumerated type describing messages passed between to/from the Runtime
/// Manager enclave (These originate from the Untrusted Pass-through (Sinaloa)
/// These messages are inteded to be serialized using bincode before transport,
/// and deserialized using bincode after transport
#[derive(Serialize, Deserialize, Debug)]
pub enum RuntimeManagerMessage {
    // A message generated by an operation that did not return data, but did
    /// return a status.
    /// Most operations return data, but if they fail, they will return a
    /// status set to `NitroStatus::Fail` (or `NitroStatus::Unimplemented` if
    /// it is not implmeneted).
    /// Parameters:
    /// NitroStatus - the Status
    Status(NitroStatus),
    /// A request to initialize the Runtime Manager enclave with the provided
    /// policy
    /// parameters:
    /// String - The policy, in JSON format
    Initialize(String),  // policy_json
    /// A request for the self-signed certificate generated by the enclave.
    /// This message is necessary for allowing tests to operate without 
    /// performing the full attestation flow.
    /// This command should not be used in real environments. It does not 
    /// constitute a security risk in and of itself, but the fetched value 
    /// cannot be trusted, and thus the certificate should actually be
    /// retrieved using attestation.
    /// This request may be removed in the future.
    GetEnclaveCert,
    /// The response to the `GetEnclaveCert` message. It contains the
    /// self-signed certificate generated by the enclave
    /// Parameters:
    /// Vec<u8> - the DER encoded certificate
    EnclaveCert(Vec<u8>),
    /// A request for the randomly generated name of the enclave.
    /// This message is necessary for allowing tests to operate without 
    /// performing the full attestation flow.
    /// This command should not be used in real environments. It does not 
    /// constitute a security risk in and of itself, but the fetched value 
    /// cannot be trusted, and thus the name should actually be retrieved using
    /// attestation.
    /// This request may be removed in the future.
    GetEnclaveName,
    /// The response to the `GetEnclaveName` message. It contains the randomly
    /// generated name of the enclave
    /// Parameters:
    /// String - the name of the enclave
    EnclaveName(String),
    /// A request for a PSA Attestation (proxy) token from the Runtime Manager
    /// enclave.
    /// Parameters:
    /// Vec<u8> - The 128-bit randomly generated challenge value
    GetPSAAttestationToken(Vec<u8>),
    /// The response to the `GetPSAAttestationToken` request.
    /// Parameters:
    /// Vec<u8> - The PSA Attestation token
    /// Vec<u8> - The public key that was randomly generated by the enclave
    ///           to be used for the TLS channel communications
    /// i32     - The Device ID that he Nitro Root Enclave uses to identify
    ///           itself to the Proxy Attestation Service
    PSAAttestationToken(Vec<u8>, Vec<u8>, i32),
    /// A request to establish a new TLS session with the enclave
    NewTLSSession,
    /// The response to the `NewTLSSession` message
    /// Parameters:
    /// u32 - The Session ID of the created TLS Session
    TLSSession(u32),
    /// A request to close an already established TLS session
    /// Parameters:
    /// u32 - The Session ID of the session to be closed
    CloseTLSSession(u32),
    /// Request to determine if the TLS Session needs data to be sent to it
    /// Parameters:
    /// u32 - The Session ID of the TLS session
    GetTLSDataNeeded(u32),
    /// Response to `GetTLSDataNeeded` message
    /// Parameters:
    /// bool - is data needed?
    TLSDataNeeded(bool),
    /// Request to send TLS data to the enclave
    /// Parameters:
    /// u32 - the Session ID of the TLS Session associated with the data
    /// Vec<u8> - The TLS Data
    SendTLSData(u32, Vec<u8>),
    /// Request TLS Data from the enclave
    /// Parameters:
    /// u32 - the Session ID of the TLS Session to request data from
    GetTLSData(u32),
    /// Response to `GetTLSData`
    /// Parameters:
    /// Vec<u8> - The TLS Data. May be empty
    /// bool    - a flag indicating if the TLS session is still alive
    TLSData(Vec<u8>, bool),    // TLS Data, alive_flag
    /// A request to reset the enclave
    ResetEnclave,
}

/// a enumerated type for Veracruz-specific socket errors
#[derive(Debug, Error)]
pub enum VeracruzSocketError {
    /// An error was returned by nix
    #[error(display = "VeracruzSocketError: Nix Error: {:?}", _0)]
    NixError(#[error(source)] nix::Error),
}

/// Send a buffer of data (using a length, buffer protocol) to the file 
/// descriptor `fd`
pub fn send_buffer(fd: RawFd, buffer: &Vec<u8>) -> Result<(), VeracruzSocketError> {
    let len = buffer.len();
    // first, send the length of the buffer
    {
        let mut buf = [0u8; 9];
        LittleEndian::write_u64(&mut buf, buffer.len() as u64);
        let mut sent_bytes = 0;
        while sent_bytes < buf.len() {
            sent_bytes += match send(fd, &buf[sent_bytes..buf.len()], MsgFlags::empty()) {
                Ok(size) => size,
                Err(err) => {
                    return Err(VeracruzSocketError::NixError(err));
                }
            };
        }
    }
    // next, send the buffer
    {
        let mut sent_bytes = 0;
        while sent_bytes < len {
            let size = match send(fd, &buffer[sent_bytes..len], MsgFlags::empty()) {
                Ok(size) => size,
                Err(nix::Error::Sys(_)) => 0,
                Err(err) => {
                    return Err(VeracruzSocketError::NixError(err));
                }
            };
            sent_bytes += size;
        }
    }
    return Ok(());
}

/// Read a buffer of data (using a length, buffer protocol) from the file 
/// descriptor `fd`
pub fn receive_buffer(fd: RawFd) -> Result<Vec<u8>, VeracruzSocketError> {
    // first, read the length
    let length = {
        let mut buf = [0u8; 9];
        let len = buf.len();
        let mut received_bytes = 0;
        while received_bytes < len {
            received_bytes += match recv(fd, &mut buf[received_bytes..len], MsgFlags::empty()) {
                Ok(size) => size,
                Err(nix::Error::Sys(EINTR)) => 0,
                Err(err) => {
                    println!("I have experienced an error:{:?}", err);
                    return Err(VeracruzSocketError::NixError(err));
                }
            }
        }
        LittleEndian::read_u64(&buf) as usize
    };
    let mut buffer: Vec<u8> = vec![0; length];
    // next, read the buffer
    {
        let mut received_bytes: usize = 0;
        while received_bytes < length {
            received_bytes += match recv(fd, &mut buffer[received_bytes..length], MsgFlags::empty())
            {
                Ok(size) => size,
                Err(nix::Error::Sys(EINTR)) => 0,
                Err(err) => {
                    return Err(VeracruzSocketError::NixError(err));
                }
            }
        }
    }
    return Ok(buffer);
}
