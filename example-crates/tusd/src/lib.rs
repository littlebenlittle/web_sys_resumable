extern crate console_error_panic_hook;
extern crate wasm_bindgen;

use anyhow::Context;
use std::panic;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Blob;
use web_sys_resumable::ResumableUpload;

macro_rules! log {
    ($($t:tt)*) => (web_sys::console::log_1(
        &JsValue::from(
            format_args!($($t)*).to_string()
        )
    ))
}

#[wasm_bindgen]
pub fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_bindgen_futures::spawn_local(async move {
        let file = {
            let str_seq = js_sys::Array::new();
            str_seq.push(&JsValue::from_str(&"blah".repeat(5)));
            web_sys::File::new_with_str_sequence(&str_seq, "blah.txt").unwrap()
        };
        let href = "http://localhost:1080/files/";
        log!("creating new upload at {}", href);
        let (mut upload, location) = new_upload(&file, href, 3).await.unwrap();
        log!("successfully created at {}", location);
        log!("uploading content");
        continue_upload(&mut upload, &location).await.unwrap();
        log!("content uploaded successfully!");
    });
}

/// Registers a new resumable upload with the remote located at `href`.
/// Returns the resumable upload metadata.
async fn new_upload<'a>(
    file: &'a web_sys::File,
    href: &str,
    chunk_sz: i32,
) -> anyhow::Result<(ResumableUpload<'a>, String)> {
    let upload = match ResumableUpload::new(file, chunk_sz).await {
        Ok(u) => u,
        Err(e) => anyhow::bail!(e.as_string().unwrap()),
    };
    log!("upload size: {}", file.size());
    log!("number of chunks: {}", (file.size() as i32 / chunk_sz));
    let res = gloo_net::http::Request::post(href)
        .header("Content-Length", "0")
        .header("Upload-Length", upload.size().to_string().as_str())
        .header("Tus-Resumable", "1.0.0")
        // TODO include filename and content hash
        // .header("Upload-Metadata", format!("filename {}", base64!(file.name()))
        .header("Content-Type", "application/offset+octet-stream")
        .send()
        .await?;
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
async fn continue_upload<'a>(
    upload: &'a mut ResumableUpload<'a>,
    location: &str,
) -> anyhow::Result<()> {
    let chunk_sz = upload.chunk_size();
    let nchunks = upload.nchunks();
    for (chunk, index) in upload.iter_unsent() {
        let offset = if index < nchunks {
            index * chunk_sz
        } else {
            chunk.size() as i32
        };
        let arr = match JsFuture::from(chunk.array_buffer()).await {
            Ok(u) => u,
            Err(e) => anyhow::bail!(e.as_string().unwrap()),
        };
        log!(
            "uploading chunk {}/{} ({}): {}",
            index + 1,
            nchunks,
            chunk.size(),
            blob_text(&chunk).await
        );
        let res = gloo_net::http::Request::patch(location)
            .header("Content-Length", chunk.size().to_string().as_str())
            .header("Upload-Offset", offset.to_string().as_str())
            .header("Content-Type", "application/offset+octet-stream")
            .header("Tus-Resumable", "1.0.0")
            .body(arr)
            .context("error setting request body")?
            .send()
            .await
            .context("error sending request")?;
        if res.status() != 204 {
            anyhow::bail!("bad response");
        }
    }
    Ok(())
}

async fn blob_text(blob: &Blob) -> String {
    JsFuture::from(
        web_sys::Response::new_with_opt_blob(Some(blob))
            .unwrap()
            .text()
            .unwrap(),
    )
    .await
    .unwrap()
    .as_string()
    .unwrap()
}
