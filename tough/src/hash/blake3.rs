use crate::hash::HashContext;

pub struct Blake3HashContext {
    inner: blake3::Hasher,
}

impl Blake3HashContext {
    pub fn new() -> Self {
        Self {
            inner: blake3::Hasher::new(),
        }
    }
}

impl HashContext for Blake3HashContext {
    fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    fn finish(self: Box<Self>) -> Vec<u8> {
        self.inner.finalize().as_bytes().to_vec()
    }

    fn name(&self) -> &str {
        "blake3"
    }
}
