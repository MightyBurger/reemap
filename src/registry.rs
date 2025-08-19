// Copyright 2025 Jordan Johnson
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Functions to register and un-register Reemap to start on login.
//!
//! Reemap is intended to be installed per-user without admin privileges. Whether Reemap starts
//! on login should be configurable per-user, not per-machine. So, the appropriate registry key
//! to modify is:
//!
//! ```text
//! HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run
//! ```
//!
//! as opposed to the one inside HKEY_LOCAL_MACHINE.
//!
//! These functions add and remove the following value to this key:
//!
//! | Name                                                        | Value                                   |
//! | :---------------------------------------------------------- | :-------------------------------------- |
//! | ```ReemapAutoLaunch_e4e2530c-65ff-4fbb-929d-1b256be9148e``` | ```<path to executable> --background``` |
//!
//! Be careful here. Accessing the Windows registry is to be done responsibly.

use thiserror::Error;
const REEMAP_RUN_VALUE_NAME: &str = "ReemapAutoLaunch_e4e2530c-65ff-4fbb-929d-1b256be9148e";
const RUN_KEY_PATH: &str = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run";

/// **This function modifies the Windows registry.** ⚠️
/// Add to the `Run` registry key to cause Reemap to run when the user logs in.
/// If the previous value exists, it is overwritten in this step.
pub fn register_run_on_login() -> Result<()> {
    use windows_registry::CURRENT_USER;

    // Determine the value to write
    let reem_run = {
        let reem_path = std::env::current_exe()?;
        let Some(reem_path) = reem_path.to_str() else {
            return Err(RegistryResult::ExePathUTF8Error);
        };
        format!("{reem_path} --background")
    };

    // Add the value to the registry key
    let run_key = CURRENT_USER.options().read().write().open(RUN_KEY_PATH)?;
    run_key.set_string(REEMAP_RUN_VALUE_NAME, reem_run)?;

    Ok(())
}

/// **This function modifies the Windows registry.** ⚠️
/// Remove from the `Run` registry key so Reemap no longer runs when the user logs in.
/// Takes no action if the registry key does not contain the value already.
pub fn unregister_run_on_login() -> Result<()> {
    use windows_registry::CURRENT_USER;

    // Check if it's registered, and if so, remove it from the registry.

    // I acknowledge the small possibility of a condition where the registry value is
    // set or removed between this check and the removal.
    // This is acceptable. Even if this happens, adding or removing this registry key should never
    // cause damage to the user's setup.

    let run_key = CURRENT_USER.options().read().write().open(RUN_KEY_PATH)?;
    let registered = run_key
        .values()?
        .any(|(name, _)| name == REEMAP_RUN_VALUE_NAME);

    if registered {
        run_key.remove_value(REEMAP_RUN_VALUE_NAME)?;
    }

    Ok(())
}

/// **This function modifies the Windows registry.** ⚠️
/// Set whether to run Reemap on login.
pub fn run_on_login(run: bool) -> Result<()> {
    if run {
        register_run_on_login()
    } else {
        unregister_run_on_login()
    }
}

/// Check the Windows registry to see if Reemap is scheduled to run on login.
pub fn is_registered_run_on_login() -> Result<bool> {
    use windows_registry::CURRENT_USER;

    let run_key = CURRENT_USER.options().read().open(RUN_KEY_PATH)?;
    Ok(run_key
        .values()?
        .any(|(name, _)| name == REEMAP_RUN_VALUE_NAME))
}

/// Possible errors from attempting to access the registry.
#[derive(Debug, Error)]
pub enum RegistryResult {
    #[error("error determining executable path: {0}")]
    ExePathIOError(#[from] std::io::Error),

    #[error("executable path contained invalid UTF-8")]
    ExePathUTF8Error,

    #[error("error modifying registry key: {0}")]
    WinRegError(#[from] windows_result::Error),
}

/// The return value of functions that access the Windows registry in Reemap.
pub type Result<T> = core::result::Result<T, RegistryResult>;
