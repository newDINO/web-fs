An asynchronous file system in webassembly based on [File System API](https://developer.mozilla.org/en-US/docs/Web/API/File_System_API), 
using a [FileSystemSyncAccessHandle](https://developer.mozilla.org/en-US/docs/Web/API/FileSystemSyncAccessHandle) running in a web worker.

Aim to be compatible with [`async-fs`]. 
But due to the restrictions of *File System API*, the API of this crate is not perfectly aligned with that of [`async-fs`].

[`async-fs`]: https://docs.rs/async-fs

File an issue if you find anything wrong. Pull requests are also welcomed.

## Limitations
This crate currently doesn't work on safari due to the reason that `FileSystemFileHandle` can't be posted to web worker.

## Maximum file size
Due to the reason that *File System API* uses *number*(f64 in Rust) to represent file size, theoretically the max file size allowed is 2<sup>53</sup>, 
or 9_007_199_254_740_992 or 8EB or 8192TB which is larger than the single file size limit of many file systems. 
So no need to worry about this.

Note that wasm32 only support at most 4GB memory, so you can't use read_to_end() for files larger than that.


## Example: Read & Write
```rust
// provides functionalities like write_all() and read_to_string()
use futures_lite::AsyncWriteExt;
use futures_lite::AsyncReadExt;

// Use web_fs in wasm and async_fs on native.
#[cfg(target_arch = "wasm32")]
use web_fs::{File, read_to_string, write, OpenOptions};
#[cfg(not(target_arch = "wasm32"))]
use async_fs::{File, read_to_string, write, OpenOptions};

// write
{
    let mut file = File::create("testf").await.unwrap();
    file.write_all("Hello,".as_bytes()).await.unwrap();
}
// read
{
    let mut file = File::open("testf").await.unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).await.unwrap();
    assert_eq!("Hello,", buf);
}
// append
{
    let mut file = OpenOptions::new().write(true).append(true).open("testf").await.unwrap();
    file.write_all(" world!".as_bytes()).await.unwrap();
}
{
    let buf = read_to_string("testf").await.unwrap();
    assert_eq!("Hello, world!", buf);
}
// convenient read write
{
    write("testf", "Hello, FS!").await.unwrap();
    let buf = read_to_string("testf").await.unwrap();
    assert_eq!("Hello, FS!", buf);
}
```
## Example: Print the content of the fs recursively
```rust
use futures_lite::StreamExt;
use wasm_bindgen::prelude::*;
use log::info;
use web_fs::{create_dir, create_dir_all, read_dir};

#[wasm_bindgen(start)]
pub async fn run() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    create_dir("test_dir1").await.unwrap();
    create_dir_all("test_dir2/child").await.unwrap();

    let mut fs_log = "fs:\n".to_owned();
    print_dir_recursively("", 0, &mut fs_log).await;
    info!("{}", fs_log);
}

async fn print_dir_recursively<P: AsRef<std::path::Path>>(path: P, level: usize, output: &mut impl std::fmt::Write) {
    let mut dir = read_dir(path).await.unwrap();
    while let Some(entry) = dir.next().await {
        let entry = entry.unwrap();
        writeln!(output, "{}{:?}", " ".repeat(level * 4), entry).unwrap();
        if entry.file_type().await.unwrap().is_dir() {
            Box::pin(print_dir_recursively(entry.path(), level + 1, output)).await;
        }
    }
}
```