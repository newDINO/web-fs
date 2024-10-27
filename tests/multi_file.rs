use futures_lite::{AsyncReadExt, AsyncWriteExt};
use wasm_bindgen_test::*;

use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);

use web_fs::*;

#[wasm_bindgen_test]
async fn test() {
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