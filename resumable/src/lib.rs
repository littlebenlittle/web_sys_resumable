mod upload;
pub use upload::ResumableUpload;

#[cfg(test)]
mod tests {

    use super::ResumableUpload;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::wasm_bindgen_test;

    fn blah_file(num: usize) -> web_sys::File {
        let file_bits = js_sys::Array::new();
        file_bits.push(&JsValue::from_str(&"blah".repeat(num)));
        web_sys::File::new_with_str_sequence(&file_bits, "blah.txt").unwrap()
    }

    #[wasm_bindgen_test]
    async fn resumable() -> anyhow::Result<()> {
        let file = blah_file(5);
        let mut upload = ResumableUpload::new(&file, 4).await?;
        assert_eq!("blah.txt", upload.file_name());
        upload
            .for_each_unsent(|i, text| async move {
                assert_eq!("blah", text, "chunk {}", i);
                if i == 2 {
                    false // pretend the 2nd chunk failed to send
                } else {
                    true // pretend we sent the chunk successfully
                }
            })
            .await;
        assert_eq!(upload.chunks() - 1, upload.sent());
        upload
            .for_each_unsent(|_, text| async move {
                assert_eq!("blah", text);
                true
            })
            .await;
        assert_eq!(upload.chunks(), upload.sent());
        Ok(())
    }
}
