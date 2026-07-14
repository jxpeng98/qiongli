use std::error::Error;
use std::ffi::c_void;
use std::fmt::{self, Debug, Display, Formatter};
use std::fs::File;
use std::io;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, RawHandle};
use std::path::Path;
use std::ptr::null_mut;

const TOKEN_QUERY: u32 = 0x0008;
const TOKEN_USER_CLASS: i32 = 1;
const ACL_REVISION: u32 = 2;
const FILE_ALL_ACCESS: u32 = 0x001f_01ff;
const FILE_READ_ATTRIBUTES: u32 = 0x0000_0080;
const GENERIC_READ: u32 = 0x8000_0000;
const GENERIC_WRITE: u32 = 0x4000_0000;
const READ_CONTROL: u32 = 0x0002_0000;
const SE_DACL_PROTECTED: u16 = 0x1000;
const CREATE_NEW: u32 = 1;
const OPEN_EXISTING: u32 = 3;
const OPEN_ALWAYS: u32 = 4;
const FILE_SHARE_READ: u32 = 0x0000_0001;
const FILE_SHARE_WRITE: u32 = 0x0000_0002;
const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x0000_0010;
const FILE_ATTRIBUTE_NORMAL: u32 = 0x0000_0080;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const INVALID_HANDLE_VALUE: *mut c_void = -1_isize as *mut c_void;
const OWNER_SECURITY_INFORMATION: u32 = 0x0000_0001;
const DACL_SECURITY_INFORMATION: u32 = 0x0000_0004;
const SE_FILE_OBJECT: i32 = 1;
const ACCESS_ALLOWED_ACE_TYPE: u8 = 0;
const INHERITED_ACE: u8 = 0x10;
const MOVEFILE_REPLACE_EXISTING: u32 = 0x0000_0001;
const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;

#[cfg(test)]
const FILE_GENERIC_READ: u32 = 0x0012_0089;
#[cfg(test)]
const SECURITY_MAX_SID_SIZE: usize = 68;
#[cfg(test)]
const WIN_WORLD_SID: i32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HandleIdentity {
    pub volume_serial_number: u32,
    pub file_index: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HandleFacts {
    pub identity: HandleIdentity,
    pub number_of_links: u32,
    pub attributes: u32,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SecurityError {
    Io(io::ErrorKind),
    InsecurePermissions,
    UnsafeObject,
}

impl SecurityError {
    #[must_use]
    pub const fn io_kind(self) -> Option<io::ErrorKind> {
        match self {
            Self::Io(kind) => Some(kind),
            Self::InsecurePermissions | Self::UnsafeObject => None,
        }
    }

    fn last_os_error() -> Self {
        Self::Io(io::Error::last_os_error().kind())
    }
}

impl Debug for SecurityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, formatter)
    }
}

impl Display for SecurityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(kind) => write!(formatter, "windows-io ({kind:?})"),
            Self::InsecurePermissions => formatter.write_str("insecure-permissions"),
            Self::UnsafeObject => formatter.write_str("unsafe-object"),
        }
    }
}

impl Error for SecurityError {}

#[repr(C)]
struct SecurityAttributes {
    _length: u32,
    _security_descriptor: *mut c_void,
    _inherit_handle: i32,
}

#[repr(C)]
struct SecurityDescriptor {
    _revision: u8,
    _reserved: u8,
    _control: u16,
    _owner: *mut c_void,
    _group: *mut c_void,
    _system_acl: *mut Acl,
    _discretionary_acl: *mut Acl,
}

#[repr(C)]
struct Acl {
    _revision: u8,
    _reserved_1: u8,
    _size: u16,
    ace_count: u16,
    _reserved_2: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SidAndAttributes {
    sid: *mut c_void,
    _attributes: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct TokenUser {
    user: SidAndAttributes,
}

#[repr(C)]
struct AceHeader {
    ace_type: u8,
    ace_flags: u8,
    _ace_size: u16,
}

#[repr(C)]
struct AccessAllowedAce {
    header: AceHeader,
    mask: u32,
    sid_start: u32,
}

#[repr(C)]
struct AccessAllowedAceLayout {
    _header: [u8; 4],
    _mask: u32,
    _sid_start: u32,
}

#[repr(C)]
#[derive(Default)]
struct FileTime {
    _low_date_time: u32,
    _high_date_time: u32,
}

#[repr(C)]
#[derive(Default)]
struct ByHandleFileInformation {
    file_attributes: u32,
    _creation_time: FileTime,
    _last_access_time: FileTime,
    _last_write_time: FileTime,
    volume_serial_number: u32,
    _file_size_high: u32,
    _file_size_low: u32,
    number_of_links: u32,
    file_index_high: u32,
    file_index_low: u32,
}

struct OwnedHandle(*mut c_void);
struct OwnedLocal(*mut c_void);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: this wrapper is constructed only from an owned token handle.
        unsafe {
            close_handle(self.0);
        }
    }
}

impl Drop for OwnedLocal {
    fn drop(&mut self) {
        // SAFETY: GetSecurityInfo allocates this buffer for LocalFree.
        unsafe {
            local_free(self.0);
        }
    }
}

#[link(name = "kernel32")]
unsafe extern "system" {
    #[link_name = "CloseHandle"]
    fn close_handle(object: *mut c_void) -> i32;
    #[link_name = "CreateDirectoryW"]
    fn create_directory_w(
        path_name: *const u16,
        security_attributes: *mut SecurityAttributes,
    ) -> i32;
    #[link_name = "CreateFileW"]
    fn create_file_w(
        file_name: *const u16,
        desired_access: u32,
        share_mode: u32,
        security_attributes: *mut SecurityAttributes,
        creation_disposition: u32,
        flags_and_attributes: u32,
        template_file: *mut c_void,
    ) -> *mut c_void;
    #[link_name = "GetCurrentProcess"]
    fn get_current_process() -> *mut c_void;
    #[link_name = "GetFileInformationByHandle"]
    fn get_file_information_by_handle(
        file: *mut c_void,
        information: *mut ByHandleFileInformation,
    ) -> i32;
    #[link_name = "LocalFree"]
    fn local_free(memory: *mut c_void) -> *mut c_void;
    #[link_name = "MoveFileExW"]
    fn move_file_ex_w(existing_file_name: *const u16, new_file_name: *const u16, flags: u32)
    -> i32;
}

#[link(name = "advapi32")]
unsafe extern "system" {
    #[link_name = "AddAccessAllowedAceEx"]
    fn add_access_allowed_ace_ex(
        acl: *mut Acl,
        ace_revision: u32,
        ace_flags: u32,
        access_mask: u32,
        sid: *const c_void,
    ) -> i32;
    #[cfg(test)]
    #[link_name = "CreateWellKnownSid"]
    fn create_well_known_sid(
        sid_type: i32,
        domain_sid: *const c_void,
        sid: *mut c_void,
        sid_size: *mut u32,
    ) -> i32;
    #[link_name = "EqualSid"]
    fn equal_sid(first_sid: *const c_void, second_sid: *const c_void) -> i32;
    #[link_name = "GetAce"]
    fn get_ace(acl: *const Acl, ace_index: u32, ace: *mut *mut c_void) -> i32;
    #[link_name = "GetLengthSid"]
    fn get_length_sid(sid: *const c_void) -> u32;
    #[link_name = "GetSecurityDescriptorControl"]
    fn get_security_descriptor_control(
        security_descriptor: *const c_void,
        control: *mut u16,
        revision: *mut u32,
    ) -> i32;
    #[link_name = "GetSecurityInfo"]
    fn get_security_info(
        handle: *mut c_void,
        object_type: i32,
        requested_information: u32,
        owner: *mut *mut c_void,
        group: *mut *mut c_void,
        dacl: *mut *mut Acl,
        system_acl: *mut *mut Acl,
        security_descriptor: *mut *mut c_void,
    ) -> u32;
    #[link_name = "GetTokenInformation"]
    fn get_token_information(
        token_handle: *mut c_void,
        token_information_class: i32,
        token_information: *mut c_void,
        token_information_length: u32,
        return_length: *mut u32,
    ) -> i32;
    #[link_name = "InitializeAcl"]
    fn initialize_acl(acl: *mut Acl, acl_length: u32, acl_revision: u32) -> i32;
    #[link_name = "InitializeSecurityDescriptor"]
    fn initialize_security_descriptor(
        security_descriptor: *mut SecurityDescriptor,
        revision: u32,
    ) -> i32;
    #[link_name = "IsValidSid"]
    fn is_valid_sid(sid: *const c_void) -> i32;
    #[link_name = "OpenProcessToken"]
    fn open_process_token(
        process_handle: *mut c_void,
        desired_access: u32,
        token_handle: *mut *mut c_void,
    ) -> i32;
    #[link_name = "SetSecurityDescriptorControl"]
    fn set_security_descriptor_control(
        security_descriptor: *mut SecurityDescriptor,
        control_bits_of_interest: u16,
        control_bits_to_set: u16,
    ) -> i32;
    #[link_name = "SetSecurityDescriptorDacl"]
    fn set_security_descriptor_dacl(
        security_descriptor: *mut SecurityDescriptor,
        dacl_present: i32,
        dacl: *mut Acl,
        dacl_defaulted: i32,
    ) -> i32;
    #[link_name = "SetSecurityDescriptorOwner"]
    fn set_security_descriptor_owner(
        security_descriptor: *mut SecurityDescriptor,
        owner: *mut c_void,
        owner_defaulted: i32,
    ) -> i32;
}

pub fn create_owner_only_directory(path: &Path) -> Result<File, SecurityError> {
    with_owner_only_security_attributes(|attributes| {
        let encoded = wide_path(path)?;
        // SAFETY: path and security structures remain valid for this call.
        if unsafe { create_directory_w(encoded.as_ptr(), attributes) } == 0 {
            return Err(SecurityError::last_os_error());
        }
        match open_owner_only_directory_path(&encoded) {
            Ok(directory) => Ok(directory),
            Err(error) => {
                let _ = std::fs::remove_dir(path);
                Err(error)
            }
        }
    })
}

pub fn open_owner_only_directory(path: &Path) -> Result<File, SecurityError> {
    let path = wide_path(path)?;
    open_owner_only_directory_path(&path)
}

pub fn open_directory_no_reparse(path: &Path) -> Result<File, SecurityError> {
    let path = wide_path(path)?;
    let file = open_raw_path(
        &path,
        FILE_READ_ATTRIBUTES,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        OPEN_EXISTING,
        FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
        null_mut(),
    )?;
    validate_kind_and_links(&file, true)?;
    Ok(file)
}

pub fn create_owner_only_new_file(path: &Path) -> Result<File, SecurityError> {
    with_owner_only_security_attributes(|attributes| {
        let path = wide_path(path)?;
        let file = open_raw_path(
            &path,
            GENERIC_READ | GENERIC_WRITE | READ_CONTROL,
            0,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT,
            attributes,
        )?;
        validate_owner_only_file(&file)?;
        Ok(file)
    })
}

pub fn open_or_create_owner_only_lock(path: &Path) -> Result<File, SecurityError> {
    with_owner_only_security_attributes(|attributes| {
        let path = wide_path(path)?;
        let file = open_raw_path(
            &path,
            GENERIC_READ | GENERIC_WRITE | READ_CONTROL,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            OPEN_ALWAYS,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT,
            attributes,
        )?;
        validate_owner_only_file(&file)?;
        Ok(file)
    })
}

pub fn open_owner_only_file(path: &Path) -> Result<File, SecurityError> {
    let path = wide_path(path)?;
    let file = open_raw_path(
        &path,
        GENERIC_READ | READ_CONTROL,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT,
        null_mut(),
    )?;
    validate_owner_only_file(&file)?;
    Ok(file)
}

pub fn handle_facts(file: &File) -> Result<HandleFacts, SecurityError> {
    let mut information = ByHandleFileInformation::default();
    // SAFETY: the borrowed file handle is valid and information is writable.
    if unsafe {
        get_file_information_by_handle(file.as_raw_handle().cast::<c_void>(), &mut information)
    } == 0
    {
        return Err(SecurityError::last_os_error());
    }
    Ok(HandleFacts {
        identity: HandleIdentity {
            volume_serial_number: information.volume_serial_number,
            file_index: (u64::from(information.file_index_high) << 32)
                | u64::from(information.file_index_low),
        },
        number_of_links: information.number_of_links,
        attributes: information.file_attributes,
    })
}

pub fn move_file_write_through(
    source: &Path,
    destination: &Path,
    replace_existing: bool,
) -> Result<(), SecurityError> {
    let source = wide_path(source)?;
    let destination = wide_path(destination)?;
    let mut flags = MOVEFILE_WRITE_THROUGH;
    if replace_existing {
        flags |= MOVEFILE_REPLACE_EXISTING;
    }
    // SAFETY: both paths are null-terminated and live for this call.
    if unsafe { move_file_ex_w(source.as_ptr(), destination.as_ptr(), flags) } == 0 {
        Err(SecurityError::last_os_error())
    } else {
        Ok(())
    }
}

fn open_owner_only_directory_path(path: &[u16]) -> Result<File, SecurityError> {
    let directory = open_raw_path(
        path,
        FILE_READ_ATTRIBUTES | READ_CONTROL,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        OPEN_EXISTING,
        FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
        null_mut(),
    )?;
    validate_kind_and_links(&directory, true)?;
    verify_owner_only_handle(directory.as_raw_handle())?;
    Ok(directory)
}

fn validate_owner_only_file(file: &File) -> Result<(), SecurityError> {
    validate_kind_and_links(file, false)?;
    verify_owner_only_handle(file.as_raw_handle())
}

fn validate_kind_and_links(file: &File, expect_directory: bool) -> Result<(), SecurityError> {
    let facts = handle_facts(file)?;
    let is_directory = facts.attributes & FILE_ATTRIBUTE_DIRECTORY != 0;
    if facts.attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || is_directory != expect_directory
        || (!expect_directory && facts.number_of_links != 1)
    {
        return Err(SecurityError::UnsafeObject);
    }
    Ok(())
}

fn open_raw_path(
    path: &[u16],
    desired_access: u32,
    share_mode: u32,
    creation_disposition: u32,
    flags: u32,
    security_attributes: *mut SecurityAttributes,
) -> Result<File, SecurityError> {
    // SAFETY: path is null-terminated and security_attributes is either null or points to
    // structures retained by the caller for the duration of this call.
    let handle = unsafe {
        create_file_w(
            path.as_ptr(),
            desired_access,
            share_mode,
            security_attributes,
            creation_disposition,
            flags,
            null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(SecurityError::last_os_error());
    }
    // SAFETY: CreateFileW returned one owned handle which is transferred to File.
    Ok(unsafe { File::from_raw_handle(handle) })
}

fn with_owner_only_security_attributes<T>(
    operation: impl FnOnce(*mut SecurityAttributes) -> Result<T, SecurityError>,
) -> Result<T, SecurityError> {
    with_current_user_sid(|sid| {
        // SAFETY: sid was validated from TOKEN_USER and remains live in token storage.
        let sid_length = unsafe { get_length_sid(sid) } as usize;
        if sid_length == 0 {
            return Err(SecurityError::last_os_error());
        }
        let acl_length = size_of::<Acl>()
            .checked_add(size_of::<AccessAllowedAceLayout>() - size_of::<u32>())
            .and_then(|length| length.checked_add(sid_length))
            .ok_or(SecurityError::Io(io::ErrorKind::InvalidData))?;
        let acl_words = acl_length.div_ceil(size_of::<u32>());
        let mut acl_storage = vec![0_u32; acl_words];
        let acl = acl_storage.as_mut_ptr().cast::<Acl>();
        let acl_storage_length = u32::try_from(acl_words * size_of::<u32>())
            .map_err(|_| SecurityError::Io(io::ErrorKind::InvalidData))?;

        // SAFETY: acl_storage is aligned, writable, and retained through operation.
        if unsafe { initialize_acl(acl, acl_storage_length, ACL_REVISION) } == 0
            || unsafe { add_access_allowed_ace_ex(acl, ACL_REVISION, 0, FILE_ALL_ACCESS, sid) } == 0
        {
            return Err(SecurityError::last_os_error());
        }

        let mut descriptor = SecurityDescriptor {
            _revision: 0,
            _reserved: 0,
            _control: 0,
            _owner: null_mut(),
            _group: null_mut(),
            _system_acl: null_mut(),
            _discretionary_acl: null_mut(),
        };
        // SAFETY: descriptor, sid, and ACL remain valid until operation returns.
        if unsafe { initialize_security_descriptor(&mut descriptor, 1) } == 0
            || unsafe { set_security_descriptor_owner(&mut descriptor, sid, 0) } == 0
            || unsafe { set_security_descriptor_dacl(&mut descriptor, 1, acl, 0) } == 0
            || unsafe {
                set_security_descriptor_control(
                    &mut descriptor,
                    SE_DACL_PROTECTED,
                    SE_DACL_PROTECTED,
                )
            } == 0
        {
            return Err(SecurityError::last_os_error());
        }

        let mut attributes = SecurityAttributes {
            _length: size_of::<SecurityAttributes>() as u32,
            _security_descriptor: (&mut descriptor as *mut SecurityDescriptor).cast(),
            _inherit_handle: 0,
        };
        operation(&mut attributes)
    })
}

fn with_current_user_sid<T>(
    operation: impl FnOnce(*mut c_void) -> Result<T, SecurityError>,
) -> Result<T, SecurityError> {
    let mut token = null_mut();
    // SAFETY: token is writable and GetCurrentProcess returns a valid pseudo-handle.
    if unsafe { open_process_token(get_current_process(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(SecurityError::last_os_error());
    }
    let _token = OwnedHandle(token);

    let mut required_length = 0_u32;
    // SAFETY: this size query intentionally supplies no output buffer.
    unsafe {
        get_token_information(token, TOKEN_USER_CLASS, null_mut(), 0, &mut required_length);
    }
    if required_length < size_of::<TokenUser>() as u32 {
        return Err(SecurityError::last_os_error());
    }
    let word_count = (required_length as usize).div_ceil(size_of::<usize>());
    let mut token_storage = vec![0_usize; word_count];
    // SAFETY: storage is aligned and sized from GetTokenInformation.
    if unsafe {
        get_token_information(
            token,
            TOKEN_USER_CLASS,
            token_storage.as_mut_ptr().cast(),
            required_length,
            &mut required_length,
        )
    } == 0
    {
        return Err(SecurityError::last_os_error());
    }
    // SAFETY: successful TOKEN_USER query initialized the leading structure.
    let token_user = unsafe { &*token_storage.as_ptr().cast::<TokenUser>() };
    // SAFETY: the SID pointer is provided by TOKEN_USER and remains in token_storage.
    if token_user.user.sid.is_null() || unsafe { is_valid_sid(token_user.user.sid) } == 0 {
        return Err(SecurityError::last_os_error());
    }
    operation(token_user.user.sid)
}

fn verify_owner_only_handle(handle: RawHandle) -> Result<(), SecurityError> {
    let mut owner = null_mut();
    let mut dacl = null_mut();
    let mut descriptor = null_mut();
    // SAFETY: output pointers are writable and handle is borrowed from a live File.
    let status = unsafe {
        get_security_info(
            handle.cast::<c_void>(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner,
            null_mut(),
            &mut dacl,
            null_mut(),
            &mut descriptor,
        )
    };
    if status != 0 {
        return Err(SecurityError::Io(
            io::Error::from_raw_os_error(status as i32).kind(),
        ));
    }
    if descriptor.is_null() || owner.is_null() || dacl.is_null() {
        return Err(SecurityError::InsecurePermissions);
    }
    let _descriptor = OwnedLocal(descriptor);

    let mut control = 0_u16;
    let mut revision = 0_u32;
    // SAFETY: descriptor is the live GetSecurityInfo-owned allocation.
    if unsafe { get_security_descriptor_control(descriptor, &mut control, &mut revision) } == 0
        || control & SE_DACL_PROTECTED == 0
    {
        return Err(SecurityError::InsecurePermissions);
    }
    // SAFETY: dacl points into the descriptor-owned allocation.
    let acl = unsafe { &*dacl };
    if acl.ace_count != 1 {
        return Err(SecurityError::InsecurePermissions);
    }
    let mut ace = null_mut();
    // SAFETY: dacl is valid and contains exactly one ACE.
    if unsafe { get_ace(dacl, 0, &mut ace) } == 0 || ace.is_null() {
        return Err(SecurityError::InsecurePermissions);
    }
    // SAFETY: GetAce returned a pointer into the descriptor-owned ACL.
    let allowed = unsafe { &*ace.cast::<AccessAllowedAce>() };
    if allowed.header.ace_type != ACCESS_ALLOWED_ACE_TYPE
        || allowed.header.ace_flags & INHERITED_ACE != 0
        || allowed.mask != FILE_ALL_ACCESS
    {
        return Err(SecurityError::InsecurePermissions);
    }
    let ace_sid = (&allowed.sid_start as *const u32).cast::<c_void>();
    with_current_user_sid(|current_user| {
        // SAFETY: all SIDs are validated and remain live for these comparisons.
        if unsafe { equal_sid(owner, current_user) } == 0
            || unsafe { equal_sid(ace_sid, current_user) } == 0
        {
            Err(SecurityError::InsecurePermissions)
        } else {
            Ok(())
        }
    })
}

fn wide_path(path: &Path) -> Result<Vec<u16>, SecurityError> {
    let mut encoded = path.as_os_str().encode_wide().collect::<Vec<_>>();
    if encoded.contains(&0) {
        return Err(SecurityError::Io(io::ErrorKind::InvalidInput));
    }
    encoded.push(0);
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    static NEXT_TEST: AtomicU64 = AtomicU64::new(0);

    struct Fixture(PathBuf);

    impl Fixture {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |duration| duration.as_nanos());
            let path = std::env::temp_dir().join(format!(
                "qiongli-windows-security-{name}-{}-{nonce}-{}",
                std::process::id(),
                NEXT_TEST.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn directories_and_files_use_a_protected_current_user_only_dacl() {
        let fixture = Fixture::new("dacl");
        let directory_path = fixture.0.join("private");
        let directory = create_owner_only_directory(&directory_path).unwrap();
        assert_ne!(
            handle_facts(&directory).unwrap().attributes & FILE_ATTRIBUTE_DIRECTORY,
            0
        );
        drop(directory);
        open_owner_only_directory(&directory_path).unwrap();

        let file_path = directory_path.join("settings.json");
        let mut file = create_owner_only_new_file(&file_path).unwrap();
        file.write_all(b"owner-only\n").unwrap();
        file.sync_all().unwrap();
        drop(file);
        open_owner_only_file(&file_path).unwrap();
    }

    #[test]
    fn broad_dacl_is_rejected() {
        let fixture = Fixture::new("broad");
        let path = fixture.0.join("broad.json");
        create_broad_acl_file(&path).unwrap();
        assert_eq!(
            open_owner_only_file(&path).unwrap_err(),
            SecurityError::InsecurePermissions
        );
    }

    #[test]
    fn hard_link_alias_is_rejected() {
        let fixture = Fixture::new("hard-link");
        let path = fixture.0.join("settings.json");
        drop(create_owner_only_new_file(&path).unwrap());
        std::fs::hard_link(&path, fixture.0.join("alias.json")).unwrap();
        assert_eq!(
            open_owner_only_file(&path).unwrap_err(),
            SecurityError::UnsafeObject
        );
    }

    #[test]
    fn write_through_replacement_moves_owner_only_bytes() {
        let fixture = Fixture::new("replace");
        let destination = fixture.0.join("settings.json");
        let source = fixture.0.join("staging.json");
        let mut first = create_owner_only_new_file(&destination).unwrap();
        first.write_all(b"first\n").unwrap();
        first.sync_all().unwrap();
        drop(first);
        let mut second = create_owner_only_new_file(&source).unwrap();
        second.write_all(b"second\n").unwrap();
        second.sync_all().unwrap();
        drop(second);

        move_file_write_through(&source, &destination, true).unwrap();
        assert!(!source.exists());
        let mut live = open_owner_only_file(&destination).unwrap();
        let mut bytes = Vec::new();
        live.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"second\n");
    }

    fn create_broad_acl_file(path: &Path) -> Result<(), SecurityError> {
        with_current_user_sid(|current_user| {
            let everyone_words = SECURITY_MAX_SID_SIZE.div_ceil(size_of::<usize>());
            let mut everyone_storage = vec![0_usize; everyone_words];
            let mut everyone_length = SECURITY_MAX_SID_SIZE as u32;
            // SAFETY: storage is aligned and large enough for a well-known SID.
            if unsafe {
                create_well_known_sid(
                    WIN_WORLD_SID,
                    null_mut(),
                    everyone_storage.as_mut_ptr().cast(),
                    &mut everyone_length,
                )
            } == 0
            {
                return Err(SecurityError::last_os_error());
            }
            let everyone = everyone_storage.as_mut_ptr().cast::<c_void>();
            // SAFETY: both SIDs are validated and live.
            let current_length = unsafe { get_length_sid(current_user) } as usize;
            // SAFETY: CreateWellKnownSid initialized everyone.
            let everyone_length = unsafe { get_length_sid(everyone) } as usize;
            let ace_prefix = size_of::<AccessAllowedAceLayout>() - size_of::<u32>();
            let acl_length = size_of::<Acl>()
                .checked_add(ace_prefix + current_length)
                .and_then(|length| length.checked_add(ace_prefix + everyone_length))
                .ok_or(SecurityError::Io(io::ErrorKind::InvalidData))?;
            let words = acl_length.div_ceil(size_of::<u32>());
            let mut storage = vec![0_u32; words];
            let acl = storage.as_mut_ptr().cast::<Acl>();
            let storage_length = u32::try_from(words * size_of::<u32>())
                .map_err(|_| SecurityError::Io(io::ErrorKind::InvalidData))?;
            // SAFETY: ACL storage and both SID pointers remain live through CreateFileW.
            if unsafe { initialize_acl(acl, storage_length, ACL_REVISION) } == 0
                || unsafe {
                    add_access_allowed_ace_ex(acl, ACL_REVISION, 0, FILE_ALL_ACCESS, current_user)
                } == 0
                || unsafe {
                    add_access_allowed_ace_ex(acl, ACL_REVISION, 0, FILE_GENERIC_READ, everyone)
                } == 0
            {
                return Err(SecurityError::last_os_error());
            }
            let mut descriptor = SecurityDescriptor {
                _revision: 0,
                _reserved: 0,
                _control: 0,
                _owner: null_mut(),
                _group: null_mut(),
                _system_acl: null_mut(),
                _discretionary_acl: null_mut(),
            };
            // SAFETY: descriptor, ACL, and SIDs remain live through CreateFileW.
            if unsafe { initialize_security_descriptor(&mut descriptor, 1) } == 0
                || unsafe { set_security_descriptor_owner(&mut descriptor, current_user, 0) } == 0
                || unsafe { set_security_descriptor_dacl(&mut descriptor, 1, acl, 0) } == 0
            {
                return Err(SecurityError::last_os_error());
            }
            let mut attributes = SecurityAttributes {
                _length: size_of::<SecurityAttributes>() as u32,
                _security_descriptor: (&mut descriptor as *mut SecurityDescriptor).cast(),
                _inherit_handle: 0,
            };
            let path = wide_path(path)?;
            let file = open_raw_path(
                &path,
                GENERIC_READ | GENERIC_WRITE | READ_CONTROL,
                0,
                CREATE_NEW,
                FILE_ATTRIBUTE_NORMAL,
                &mut attributes,
            )?;
            drop(file);
            Ok(())
        })
    }
}
