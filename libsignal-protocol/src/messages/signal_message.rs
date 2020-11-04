use crate::{
    errors::{Error, InternalError},
    keys::PublicKey,
    messages::{CiphertextMessage, CiphertextType},
    raw_ptr::Raw,
    Context, ContextInner,
};
use std::{convert::TryFrom, rc::Rc};

// For rustdoc link resolution
#[allow(unused_imports)]
use crate::keys::IdentityKeyPair;

/// A message with an arbitrary payload.
#[derive(Debug, Clone)]
pub struct SignalMessage {
    pub(crate) raw: Raw<sys::signal_message>,
    pub(crate) _ctx: Rc<ContextInner>,
}

impl SignalMessage {
    /// Is this a legacy message?
    pub fn is_legacy(msg: &[u8]) -> bool {
        unsafe { sys::signal_message_is_legacy(msg.as_ptr(), msg.len()) != 0 }
    }

    /// Get the public half of the sender's [`IdentityKeyPair`].
    pub fn sender_ratchet_key(&self) -> PublicKey {
        unsafe {
            let raw = sys::signal_message_get_sender_ratchet_key(
                self.raw.as_const_ptr(),
            );
            PublicKey {
                raw: Raw::copied_from(raw),
            }
        }
    }

    /// The message format version.
    pub fn message_version(&self) -> u8 {
        unsafe {
            sys::signal_message_get_message_version(self.raw.as_const_ptr())
        }
    }

    /// Get the signal message counter.
    pub fn counter(&self) -> u32 {
        unsafe { sys::signal_message_get_counter(self.raw.as_const_ptr()) }
    }

    /// The message body.
    pub fn body(&self) -> &[u8] {
        unsafe {
            let buffer = sys::signal_message_get_body(self.raw.as_const_ptr());
            assert!(!buffer.is_null());

            let len = sys::signal_buffer_len(buffer);
            let data = sys::signal_buffer_data(buffer);

            std::slice::from_raw_parts(data, len)
        }
    }

    /// Verify the MAC on the signal message.
    pub fn verify_mac(
        &self,
        sender_identity_key: &PublicKey,
        receiver_identity_key: &PublicKey,
        mac: &[u8],
        ctx: &Context,
    ) -> Result<bool, Error> {
        unsafe {
            let code = sys::signal_message_verify_mac(
                self.raw.as_ptr(),
                sender_identity_key.raw.as_ptr(),
                receiver_identity_key.raw.as_ptr(),
                mac.as_ptr(),
                mac.len(),
                ctx.raw(),
            );

            match code {
                0 => Ok(false),
                1 => Ok(true),
                other => Err(InternalError::from_error_code(other)
                    .unwrap_or(InternalError::Unknown)
                    .into()),
            }
        }
    }
}

impl TryFrom<CiphertextMessage> for SignalMessage {
    type Error = Error;

    fn try_from(other: CiphertextMessage) -> Result<Self, Self::Error> {
        if other.get_type()? != CiphertextType::Signal {
            Err(Error::NoSignalMessage)
        } else {
            // safety: the `CiphertextType` check tells us this is actually a
            // pointer to a `signal_message`
            let raw = unsafe {
                Raw::copied_from(other.raw.as_ptr() as *mut sys::signal_message)
            };
            Ok(SignalMessage {
                raw,
                _ctx: other._ctx,
            })
        }
    }
}

impl From<SignalMessage> for CiphertextMessage {
    fn from(other: SignalMessage) -> CiphertextMessage {
        CiphertextMessage {
            raw: other.raw.upcast(),
            _ctx: other._ctx,
        }
    }
}

impl_deserializable!(SignalMessage, signal_message_deserialize);

impl_is_a!(sys::signal_message => sys::ciphertext_message);
