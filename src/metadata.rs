use std::time::SystemTime;
use std::io::{Result, Error, ErrorKind};

use crate::FileType;

pub struct Metadata {
    pub(crate) ty: FileType,
    pub(crate) len: u64,
}

impl Metadata {
    /// Always returns Err because it is currently not supported in *File System API*.
    pub fn accsessed(&self) -> Result<SystemTime> {
        Err(Error::from(ErrorKind::Other))
    }
    /// Always returns Err because it is currently not supported in *File System API*.
    pub fn created(&self) -> Result<SystemTime> {
        Err(Error::from(ErrorKind::Other))
    }
    /// Returns the file type for this metadata.
    pub fn file_type(&self) -> FileType {
        self.ty
    }
    pub fn is_dir(&self) -> bool {
        self.ty.is_dir()
    }
    pub fn is_file(&self) -> bool {
        self.ty.is_file()
    }
    /// Always returns false because symlink is currently not supported in *File System API*.
    pub fn is_symlink(&self) -> bool {
        self.ty.is_symlink()
    }
    pub fn len(&self) -> u64 {
        self.len
    }
    /// Always returns Err because it is currently not supported in *File System API*.
    pub fn modified(&self) -> Result<SystemTime> {
        Err(Error::from(ErrorKind::Other))
    }
    pub fn permissions(&self) -> Permissions {
        Permissions {
            readonly: false,
        }
    }
}


pub struct Permissions {
    readonly: bool,
}

impl Permissions {
    pub fn readonly(&self) -> bool {
        self.readonly
    }
    pub fn set_readonly(&mut self, readonly: bool) {
        self.readonly = readonly
    }
}