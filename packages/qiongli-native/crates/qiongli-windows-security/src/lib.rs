//! Safe boundary around the small set of Win32 filesystem operations Qiongli needs.

#![cfg_attr(not(windows), forbid(unsafe_code))]

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use windows::{
    HandleFacts, HandleIdentity, SecurityError, create_owner_only_directory,
    create_owner_only_new_file, handle_facts, move_file_write_through, open_directory_no_reparse,
    open_or_create_owner_only_lock, open_owner_only_directory, open_owner_only_file,
    verify_owner_only_directory_handle, verify_owner_only_file_handle,
};
