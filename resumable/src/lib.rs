mod chunk_iter;
mod upload;

pub(crate) use chunk_iter::ChunkIter;
pub use upload::ResumableUpload;

#[macro_export]
macro_rules! unwrap_js {
    ($expr: expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => anyhow::bail!(e.as_string().unwrap()),
        }
    };
}
