// This only runs in the browser
#![cfg(target_arch = "wasm32")]

use futures_lite::{AsyncReadExt, AsyncWriteExt};
use wasm_bindgen_test::*;

use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);
wasm_bindgen_test_configure!(run_in_dedicated_worker);

use web_fs::*;

#[wasm_bindgen_test]
async fn read_write() {
    use futures_lite::AsyncReadExt;
    use futures_lite::AsyncWriteExt;
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
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("testf")
            .await
            .unwrap();
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
    // copy
    {
        copy("testf", "testf2").await.unwrap();
        assert_eq!("Hello, FS!", read_to_string("testf2").await.unwrap());
    }
    // truncate
    {
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .open("testf")
            .await
            .unwrap();
        file.set_len(5).await.unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).await.unwrap();
        assert_eq!("Hello", buf);
    }
}

#[wasm_bindgen_test]
async fn multi_file() {
    create_dir("dir1").await.unwrap();
    create_dir_all("dir2/dir1").await.unwrap();

    {
        let mut file1 = File::create("dir1/file1").await.unwrap();
        let mut file2 = File::create("dir2/dir1/file1").await.unwrap();
        file1.write_all(b"This is file1").await.unwrap();
        file2.write_all(b"This is file2").await.unwrap();
    }
    {
        let mut file1 = File::open("dir1/file1").await.unwrap();
        let mut file2 = File::open("dir2/dir1/file1").await.unwrap();
        let mut file3 = File::create("dir2/file1").await.unwrap();
        file3.write_all(b"This is file3").await.unwrap();
        let mut buf1 = String::new();
        let mut buf2 = String::new();
        file1.read_to_string(&mut buf1).await.unwrap();
        file2.read_to_string(&mut buf2).await.unwrap();
        assert_eq!("This is file1", buf1);
        assert_eq!("This is file2", buf2);
    }
    {
        let buf = read_to_string("dir2/file1").await.unwrap();
        assert_eq!("This is file3", buf);
    }
}
