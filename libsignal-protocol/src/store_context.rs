use crate::{Address, Buffer, Error, InternalError, SessionRecord, context::ContextInner, errors::FromInternalErrorCode, keys::{IdentityKeyPair, PreKey, SessionSignedPreKey}, raw_ptr::Raw};
use std::{
    fmt::{self, Debug, Formatter},
    ptr,
    rc::Rc,
};

/// Something which contains state used by the signal protocol.
///
/// Under the hood this contains several "Stores" for various keys and session
/// state (e.g. which identities are trusted, and their pre-keys).
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct StoreContext(pub(crate) Rc<StoreContextInner>);

impl StoreContext {
    pub(crate) fn new(
        raw: *mut sys::signal_protocol_store_context,
        ctx: &Rc<ContextInner>,
    ) -> StoreContext {
        StoreContext(Rc::new(StoreContextInner {
            raw,
            ctx: Rc::clone(ctx),
        }))
    }

    /// Return the identity key pair of this store.
    pub fn identity_key_pair(&self) -> Result<IdentityKeyPair, Error> {
        unsafe {
            let mut key_pair = std::ptr::null_mut();
            sys::signal_protocol_identity_get_key_pair(
                self.raw(),
                &mut key_pair,
            )
            .into_result()?;
            Ok(IdentityKeyPair {
                raw: Raw::from_ptr(key_pair),
            })
        }
    }

    /// Store pre key
    pub fn store_pre_key(&self, pre_key: &PreKey) -> Result<(), Error> {
        unsafe {
            sys::signal_protocol_pre_key_store_key(
                self.raw(),
                pre_key.raw.as_ptr(),
            )
            .into_result()?;

            Ok(())
        }
    }

    /// Store signed pre key
    pub fn store_signed_pre_key(
        &self,
        signed_pre_key: &SessionSignedPreKey,
    ) -> Result<(), Error> {
        unsafe {
            sys::signal_protocol_signed_pre_key_store_key(
                self.raw(),
                signed_pre_key.raw.as_ptr(),
            )
            .into_result()?;

            Ok(())
        }
    }

    /// Get the registration ID.
    pub fn registration_id(&self) -> Result<u32, Error> {
        unsafe {
            let mut id = 0;
            sys::signal_protocol_identity_get_local_registration_id(
                self.raw(),
                &mut id,
            )
            .into_result()?;

            Ok(id)
        }
    }

    /// Does this store already contain a session with the provided recipient?
    pub fn contains_session(&self, addr: &Address) -> Result<bool, Error> {
        unsafe {
            match sys::signal_protocol_session_contains_session(
                self.raw(),
                addr.raw(),
            ) {
                0 => Ok(false),
                1 => Ok(true),
                code => Err(InternalError::from_error_code(code)
                    .unwrap_or(InternalError::Unknown)
                    .into()),
            }
        }
    }

    /// Return the saved public identity key for a remote client.
    pub fn get_identity(&self, addr: &Address) -> Result<Option<Buffer>, Error> {
        unsafe {
            let mut raw = std::ptr::null_mut();
            sys::signal_protocol_identity_get_identity(self.raw(), addr.raw(), &mut raw).into_result()?;
            dbg!(raw);
            if raw.is_null() {
                Ok(None)
            } else {
                Ok(Some(Buffer::from_raw(raw)))
            }
        }
    }

    /// Load the session corresponding to the provided recipient.
    pub fn load_session(&self, addr: &Address) -> Result<SessionRecord, Error> {
        unsafe {
            let mut raw = ptr::null_mut();
            sys::signal_protocol_session_load_session(
                self.raw(),
                &mut raw,
                addr.raw(),
            )
            .into_result()?;

            Ok(SessionRecord {
                raw: Raw::from_ptr(raw),
                ctx: Rc::clone(&self.0.ctx),
            })
        }
    }

    /// Load the sub-device sessions corresponding to the provided recipient
    /// identifier.
    pub fn get_sub_device_sessions(
        &self,
        identifier: &str,
    ) -> Result<Vec<i32>, Error> {
        unsafe {
            let mut sessions = ptr::null_mut();
            sys::signal_protocol_session_get_sub_device_sessions(
                self.raw(),
                &mut sessions,
                identifier.as_ptr() as *const ::std::os::raw::c_char,
                identifier.len(),
            )
            .into_result()?;
            let mut ids = Vec::with_capacity(
                sys::signal_int_list_size(sessions) as usize,
            );
            for i in 0..sys::signal_int_list_size(sessions) {
                ids.push(sys::signal_int_list_at(sessions, i));
            }
            Ok(ids)
        }
    }

    /// Delete an existing session corresponding to the provided address.
    pub fn delete_session(&self, address: &Address) -> Result<(), Error> {
        unsafe {
            sys::signal_protocol_session_delete_session(
                self.raw(),
                address.raw(),
            )
            .into_result()?;
        }
        Ok(())
    }

    pub(crate) fn raw(&self) -> *mut sys::signal_protocol_store_context {
        self.0.raw
    }
}

pub(crate) struct StoreContextInner {
    raw: *mut sys::signal_protocol_store_context,
    // the global context must outlive `signal_protocol_store_context`
    #[allow(dead_code)]
    ctx: Rc<ContextInner>,
}

impl Drop for StoreContextInner {
    fn drop(&mut self) {
        unsafe {
            sys::signal_protocol_store_context_destroy(self.raw);
        }
    }
}

impl Debug for StoreContextInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("StoreContextInner").finish()
    }
}
