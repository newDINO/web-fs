use wasm_bindgen_test::*;

use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test() {
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
}