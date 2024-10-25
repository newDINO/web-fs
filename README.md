An asynchronous file system in webassembly based on *File System API*.  
Still under development, many features missing.
## Example
```rust
// provide functions like `read_to_string()` and `write_all()`
use futures_lite::AsyncWriteExt;
use futures_lite::AsyncReadExt;
// writing
{
    let mut file = OpenOptions::new().write().create().open("testf").await.unwrap();
    file.write_all("Hello, fs!".as_bytes()).await.unwrap();
}
// reading
{
    let mut file = OpenOptions::new().read().open("testf").await.unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).await.unwrap();
}

```