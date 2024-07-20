use std::future::Future;

use serde::{Deserialize, Serialize};
use sha2::Digest;
use wasm_bindgen_futures::JsFuture;
use web_sys::File;

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
    pub async fn new<'a>(file: &'a File, chunk_sz: i32) -> anyhow::Result<Self> {
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
    pub async fn enliven<'a>(self, file: &'a File) -> anyhow::Result<ResumableUpload<'a>> {
        let hash = Self::hash_parts(file).await?;
        if hash != self.hash {
            anyhow::bail!("hashes don't match")
        }
        Ok(ResumableUpload { data: self, file })
    }

    async fn hash_parts(file: &File) -> anyhow::Result<[[u8; 32]; 8]> {
        let mut hasher = sha2::Sha256::default();
        let file_sz = file.size() as i32;
        for i in 0..(file_sz / 80_000 + 1) {
            let start = i * 80_000;
            let end = (start + 80_000).min(file_sz);
            let chunk = file.slice_with_i32_and_i32(start, end).unwrap();
            match JsFuture::from(chunk.text()).await {
                Ok(v) => hasher.update(v.as_string().unwrap().as_bytes()),
                Err(e) => anyhow::bail!(e.as_string().unwrap()),
            }
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
    file: &'a File,
}

impl<'a> ResumableUpload<'a> {
    pub async fn new(file: &'a File, chunk_sz: i32) -> anyhow::Result<ResumableUpload<'a>> {
        Ok(Self {
            data: ResumableUploadData::new(file, chunk_sz).await?,
            file,
        })
    }

    pub fn file_name(&self) -> String {
        self.file.name()
    }

    pub async fn for_each_unsent<F, Fut>(&mut self, f: F)
    where
        F: Fn(i32, String) -> Fut,
        Fut: Future<Output = bool>,
    {
        let mut i = 0;
        let file_sz = self.file.size() as i32;
        for sent in &mut self.data.sent {
            if !*sent {
                let start = i * self.data.chunk_sz;
                let end = (start + self.data.chunk_sz).min(file_sz);
                let chunk = self.file.slice_with_i32_and_i32(start, end).unwrap();
                let text = JsFuture::from(chunk.text())
                    .await
                    .unwrap()
                    .as_string()
                    .unwrap();
                *sent = f(i, text).await;
            }
            i += 1;
        }
    }

    #[inline(always)]
    pub fn chunk_size(&self) -> i32 {
        self.data.chunk_sz
    }

    #[inline(always)]
    pub fn chunks(&self) -> u64 {
        self.data.sent.len() as u64
    }

    #[inline(always)]
    pub fn sent(&self) -> u64 {
        let mut n = 0;
        for sent in &self.data.sent {
            if *sent {
                n += 1
            }
        }
        return n;
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
