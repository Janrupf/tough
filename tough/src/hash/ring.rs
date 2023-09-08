use crate::hash::HashContext;

pub struct RingHashContext {
    inner: ring::digest::Context,
    name: &'static str,
}

impl RingHashContext {
    pub fn sha256() -> Self {
        Self {
            inner: ring::digest::Context::new(&ring::digest::SHA256),
            name: "sha256",
        }
    }
}

impl HashContext for RingHashContext {
    fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    fn finish(self: Box<Self>) -> Vec<u8> {
        self.inner.finish().as_ref().to_vec()
    }

    fn name(&self) -> &str {
        self.name
    }
}