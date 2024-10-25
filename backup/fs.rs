use std::{
    io::{Error, ErrorKind},
    path::Path,
};

use js_sys::{Object, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    window, FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemGetFileOptions,
    FileSystemWritableFileStream,
};

const DYN_INTO_ERROR: &str = "Javascript type dyn_into() failed";

async fn get_root() -> Result<FileSystemDirectoryHandle, Error> {
    JsFuture::from(window().unwrap().navigator().storage().get_directory())
        .await
        .map_err(|_| Error::other("Get StorageDirectory root directory failed"))?
        .dyn_into::<FileSystemDirectoryHandle>()
        .map_err(|_| Error::other(DYN_INTO_ERROR))
}

fn js_value_to_string(v: JsValue) -> String {
    format!("{}", Object::from(v).to_string())
}
fn js_value_to_error(v: JsValue) -> Error {
    Error::other(js_value_to_string(v))
}

pub struct File {
    handle: FileSystemFileHandle,
}

impl File {
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<File, Error> {
        let name = path.as_ref().to_string_lossy();
        let root = get_root().await?;
        let handle = JsFuture::from(root.get_file_handle(&name))
            .await
            .map_err(|_| Error::from(ErrorKind::PermissionDenied))?
            .dyn_into::<FileSystemFileHandle>()
            .map_err(|_| Error::other(DYN_INTO_ERROR))?;
        Ok(File { handle })
    }
    pub async fn create<P: AsRef<Path>>(path: P) -> Result<File, Error> {
        let name = path.as_ref().to_string_lossy();
        let root = get_root().await?;
        let option = FileSystemGetFileOptions::new();
        option.set_create(true);
        let handle = JsFuture::from(root.get_file_handle_with_options(&name, &option))
            .await
            .map_err(|_| Error::from(ErrorKind::PermissionDenied))?
            .dyn_into::<FileSystemFileHandle>()
            .map_err(|_| Error::other(DYN_INTO_ERROR))?;
        Ok(File { handle })
    }
    pub async fn write_all(&self, buf: &[u8]) -> Result<(), Error> {
        let stream = JsFuture::from(self.handle.create_writable())
            .await
            .map_err(|_| Error::other("Creating Writable Stream Failed"))?
            .dyn_into::<FileSystemWritableFileStream>()
            .map_err(|_| Error::other(DYN_INTO_ERROR))?;
        const WRITE_ERROR: &str = "Write failed";
        JsFuture::from(
            stream
                .write_with_u8_array(buf)
                .map_err(|_| Error::other(WRITE_ERROR))?,
        )
        .await
        .map_err(|_| Error::other(WRITE_ERROR))?;
        JsFuture::from(stream.close())
            .await
            .map_err(|e| js_value_to_error(e))?;
        Ok(())
    }
    pub async fn read_to_end(&self, buf: &mut Vec<u8>) -> Result<usize, Error> {
        let file = JsFuture::from(self.handle.get_file())
            .await
            .map_err(|e| js_value_to_error(e))?
            .dyn_into::<web_sys::File>()
            .map_err(|_| Error::other(DYN_INTO_ERROR))?;
        let array_buffer = JsFuture::from(file.array_buffer())
            .await
            .map_err(|e| js_value_to_error(e))?;
        let uint8array = Uint8Array::new(&array_buffer);
        let v = uint8array.to_vec();
        buf.extend_from_slice(&v);
        Ok(v.len())
    }
}
