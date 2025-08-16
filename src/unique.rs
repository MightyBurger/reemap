// Copyright 2025 Jordan Johnson
// SPDX-License-Identifier: Apache-2.0 OR MIT

use thiserror::Error;
use tracing::debug;
use windows::Win32::Foundation::{self as FN, ERROR_ALREADY_EXISTS};
use windows::Win32::System::Threading as TH;

/// The idea of this struct is that it can only ever be instantiated once.
/// Not once per process, but once per entire Windows login session.
/// The use case is to ensure only one instance of this programming is running at a time.
pub struct UniqueGuard(FN::HANDLE);

impl UniqueGuard {
    pub fn try_lock() -> Result<Self> {
        debug!("locking unique mutex");
        let id = windows_strings::w!("ReemapUniqueGuardMutexName");
        let handle = unsafe { TH::CreateMutexW(None, true, id)? };
        // For some bizarre reason, even if the above fails, it does not produce an Err() variant.
        // So we still need to check...
        let last_error = unsafe { FN::GetLastError() };
        if last_error == ERROR_ALREADY_EXISTS {
            return Err(Error::OtherInstanceRunning);
        }
        Ok(Self(handle))
    }
}

impl Drop for UniqueGuard {
    fn drop(&mut self) {
        debug!("releasing unique mutex");
        unsafe {
            let _ = FN::CloseHandle(self.0);
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("could not create a unique guard")]
    CannotCreateUniqueGuard(#[from] windows::core::Error),
    #[error("another instance is already running")]
    OtherInstanceRunning,
}

pub type Result<T> = std::result::Result<T, Error>;
