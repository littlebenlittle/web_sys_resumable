use base64::Engine;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys_resumable::ResumableUpload;

macro_rules! base64 {
    ($expr:expr) => {
        base64::prelude::BASE64_STANDARD.encode($expr)
    };
}

/// Registers a new resumable upload with the remote located at `href`.
/// Returns the resumable upload metadata.
pub async fn new_upload<'a>(
    file: &'a web_sys::File,
    href: &str,
    chunk_sz: i32,
    metadata: &[(&str, &str)],
) -> anyhow::Result<(ResumableUpload<'a>, String)> {
    let upload = ResumableUpload::new(file, chunk_sz).await?;
    let mut req = gloo_net::http::Request::post(href)
        .header("Content-Length", "0")
        .header("Upload-Length", upload.size().to_string().as_str())
        .header("Tus-Resumable", "1.0.0")
        .header("Content-Type", "application/offset+octet-stream");
    if metadata.len() > 0 {
        let mut metadata_kvs = Vec::new();
        for (key, val) in metadata {
            if key.contains(" ") || key.contains(",") {
                anyhow::bail!(
                    "metadata keys can't contain whitespace or commas: `{}`",
                    key
                )
            }
            metadata_kvs.push(format!("{} {}", key, base64!(val)))
        }
        req = req.header("Upload-Metadata", &metadata_kvs.join(","));
    }
    let res = req.send().await?;
    if res.status() != 201 {
        anyhow::bail!(
            "expected 201 Created, got {}: {}",
            res.status(),
            res.text().await.unwrap()
        );
    }
    let location = res.headers().get("Location").unwrap();
    return Ok((upload, location));
}

/// Continue a previously registered upload
pub async fn continue_upload<'a>(
    upload: &'a mut ResumableUpload<'a>,
    location: &str,
) -> anyhow::Result<()> {
    let chunk_sz = upload.chunk_size();
    upload
        .for_each_unsent(move |i, chunk| async move {
            let buf = JsFuture::from(chunk.array_buffer())
                .await
                .unwrap()
                .dyn_into::<js_sys::ArrayBuffer>()
                .unwrap();
            let res = gloo_net::http::Request::patch(location)
                .header("Content-Length", buf.byte_length().to_string().as_str())
                .header("Upload-Offset", (i * chunk_sz).to_string().as_str())
                .header("Content-Type", "application/offset+octet-stream")
                .header("Tus-Resumable", "1.0.0")
                .body(buf)
                .expect("error setting request body")
                .send()
                .await
                .expect("error sending request");
            if res.status() == 204 {
                true
            } else {
                false
            }
        })
        .await;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn full_upload() {
        let file = {
            let str_seq = js_sys::Array::new();
            str_seq.push(&JsValue::from_str(&"blah".repeat(5)));
            web_sys::File::new_with_str_sequence(&str_seq, "blah.txt").unwrap()
        };
        let href = "http://localhost:1080/files/";
        let (mut upload, location) = new_upload(&file, href, 3, &[("filename", &file.name())])
            .await
            .unwrap();
        continue_upload(&mut upload, &location).await.unwrap();
    }
}
