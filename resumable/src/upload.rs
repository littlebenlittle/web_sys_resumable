use js_sys::wasm_bindgen::JsValue;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use wasm_bindgen_futures::JsFuture;

use crate::{unwrap_js, ChunkIter};

// TODO BYO digest function
/// Metadata for a resumable upload. Can be serialized and
/// deserialized to restore state. Use `.enliven` to restore
/// the data backing this resumable upload .
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResumableUploadData {
    hash: [[u8; 32]; 8],
    chunk_sz: i32,
    sent: Vec<bool>,
}

impl ResumableUploadData {
    /// Create a new resumable upload from a file.
    pub async fn new<'a>(file: &'a web_sys::File, chunk_sz: i32) -> Result<Self, JsValue> {
        let file_sz = file.size() as i32;
        let mut nchunks = (file_sz) / chunk_sz;
        if file_sz % chunk_sz != 0 {
            nchunks += 1;
        }
        let sent = vec![false; nchunks as usize];
        Ok(Self {
            hash: Self::hash_parts(file).await?,
            chunk_sz,
            sent,
        })
    }

    /// Restore the data backing this resumable upload. Returns an error if
    /// the hashes do not match.
    pub async fn enliven<'a>(self, file: &'a web_sys::File) -> anyhow::Result<ResumableUpload<'a>> {
        let hash = unwrap_js!(Self::hash_parts(file).await);
        if hash != self.hash {
            anyhow::bail!("hashes don't match")
        }
        Ok(ResumableUpload { data: self, file })
    }

    async fn hash_parts(file: &web_sys::File) -> Result<[[u8; 32]; 8], JsValue> {
        let mut hasher = sha2::Sha256::default();
        for chunk in ChunkIter::new(file, 80_000) {
            let v = blob_to_vec(&chunk).await?;
            hasher.update(v)
        }
        let mut parts = [[0u8; 32]; 8];
        let mut i = 0;
        for part in hasher.finalize().chunks_exact(32) {
            parts[i] = part.try_into().unwrap();
            i += 1;
        }
        Ok(parts)
    }
}

pub struct ResumableUpload<'a> {
    data: ResumableUploadData,
    file: &'a web_sys::File,
}

impl<'a> ResumableUpload<'a> {
    pub async fn new(
        file: &'a web_sys::File,
        chunk_sz: i32,
    ) -> Result<ResumableUpload<'a>, JsValue> {
        Ok(Self {
            data: ResumableUploadData::new(file, chunk_sz).await?,
            file,
        })
    }

    pub fn iter_unsent(&'a self) -> impl Iterator<Item = (web_sys::Blob, i32)> + 'a {
        let iter = ChunkIter::new(self.file, self.data.chunk_sz);
        iter.zip(self.data.sent.iter())
            .zip(0..self.data.sent.len())
            .filter_map(move |((chunk, is_sent), index)| {
                if !is_sent {
                    Some((chunk, index as i32))
                } else {
                    None
                }
            })
    }

    #[inline(always)]
    pub fn chunk_size(&self) -> i32 {
        self.data.chunk_sz
    }

    #[inline(always)]
    pub fn nchunks(&self) -> i32 {
        self.data.sent.len() as i32
    }

    #[inline(always)]
    pub fn as_data(&self) -> ResumableUploadData {
        self.data.clone()
    }

    #[inline(always)]
    pub fn size(&self) -> i32 {
        self.file.size() as i32
    }
}

async fn blob_to_vec(blob: &web_sys::Blob) -> Result<Vec<u8>, js_sys::wasm_bindgen::JsValue> {
    let array_buffer = JsFuture::from(blob.array_buffer()).await?;
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    Ok(uint8_array.to_vec())
}
