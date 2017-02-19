//! Simple in the sense of: frames will only be decoded in whole, which will cause some latency
//! when receiving huge frames, and require more memory.
//!
//! However, this also means the type returned is easier to handle, as you will deal with only
//! complete responses.
//!
//! When sending, you will have to load everything in memory beforehand, as the frame must be
//! prepared in advance.
mod client;
mod codec;

pub use self::client::*;
pub use self::codec::*;
