An asynchronous file system in webassembly based on *File System API*.  
Aim to be compatible with async-fs.
Still under development, many features missing.  
File an issue if you find anything wrong.

## Maximum file size
Due to the reason that *File System API* uses *number*(f64 in Rust) to represent file size, the max file size allowed is 2<sup>53</sup>.

This is because the IEEE 754 double-precision floating-point format (used in f64) has a 53-bit mantissa (52 explicit bits plus 1 implicit bit),
which allows integers up to 2<sup>53</sup>-1 to be exactly represented without loss of precision.
Beyond this value, some integers may lose precision due to rounding when represented as f64, leading to potential inaccuracies during conversions back and forth.

But the test below demonstrates that 2<sup>53</sup> can also be converted between f64 and u64 correctly. So the maximum file size is actually 2<sup>53</sup>.
```rust
let a = 2f64.powi(53);
let b = a + 1.0;
let c = a - 1.0;
info!("{}, {}, {}", a as u64, b as u64, c as u64);
// 9007199254740992, 9007199254740992, 9007199254740991
let a = 2u64.pow(53);
let b = a + 1;
let c = a - 1;
info!("{}, {}, {}", a as f64, b as f64, c as f64);
// 9007199254740992, 9007199254740992, 9007199254740991
```

## Example: Read & Write
```rust
use futures_lite::AsyncWriteExt;
use futures_lite::AsyncReadExt;
use web_fs::{File, read_to_string, write, OpenOptions};
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