use crate::error::HashMismatchSnafu;
use crate::schema::Hashes;

pub mod ring;

/// Dynamic dispatch for hash algorithms.
pub trait HashContext: Send + Sync {
    /// Updates the hash context with the given data.
    fn update(&mut self, data: &[u8]);

    /// Finishes the hash context and returns the digest.
    fn finish(self: Box<Self>) -> Vec<u8>;

    /// Returns the name of the hash algorithm.
    fn name(&self) -> &str;
}

/// Generic hasher for downloading and hashing a file.
pub struct DownloadHasher {
    context: Box<dyn HashContext>,
    expected: Vec<u8>,
}

impl DownloadHasher {
    /// Constructs hashers for a all supported algorithms.
    pub fn all_supported(hashes: &Hashes) -> Vec<Self> {
        let mut out = Vec::new();

        for (alg, hash) in &hashes.values {
            match alg.as_str() {
                "sha256" => {
                    let expected = match Self::decode_hex_json(alg, hash) {
                        None => continue,
                        Some(v) => v,
                    };

                    out.push(Self::sha256(expected));
                }
                _ => {}
            }
        }

        out
    }

    fn decode_hex_json(alg: &str, value: &serde_json::Value) -> Option<Vec<u8>> {
        if let serde_json::Value::String(s) = value {
            match hex::decode(s) {
                Ok(decoded) => Some(decoded),
                Err(e) => {
                    log::warn!("Failed to decode {} hash: {}", alg, e);
                    None
                }
            }
        } else {
            log::warn!("Expected a string for {} hash, got {}", alg, value);
            None
        }
    }

    pub fn sha256(expected: Vec<u8>) -> Self {
        Self {
            context: Box::new(ring::RingHashContext::sha256()),
            expected,
        }
    }

    /// Updates the hash with the given data.
    pub fn update(&mut self, data: &[u8]) {
        self.context.update(data);
    }

    /// Validates the hash against the expected value.
    pub fn validate(self, context: &str) -> Result<(), std::io::Error> {
        let digest = self.context.finish();
        if digest.as_slice() != self.expected.as_slice() {
            HashMismatchSnafu {
                context: context.to_string(),
                calculated: hex::encode(digest),
                expected: hex::encode(&self.expected),
            }
            .fail()?;
        }

        Ok(())
    }
}
