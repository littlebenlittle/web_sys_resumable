// Iterator over chuncks of a file.
pub(crate) struct ChunkIter<'a> {
    file: &'a web_sys::Blob,
    file_sz: i32,
    chunk_sz: i32,
    i: i32,
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = web_sys::Blob;
    fn next(&mut self) -> Option<Self::Item> {
        let start = self.chunk_sz * self.i;
        if start > self.file_sz {
            return None;
        }
        self.i += 1;
        let end = (self.chunk_sz * self.i).min(self.file_sz);
        Some(self.file.slice_with_i32_and_i32(start, end).unwrap())
    }
}

impl<'a> ChunkIter<'a> {
    pub fn new(file: &'a web_sys::File, chunk_sz: i32) -> Self {
        Self {
            file,
            file_sz: file.size() as i32,
            chunk_sz,
            i: 0,
        }
    }
}
