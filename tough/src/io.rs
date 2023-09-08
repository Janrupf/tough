// Copyright 2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::error;
use crate::hash::DownloadHasher;
use crate::schema::Hashes;
use std::io::{self, Read};
use url::Url;

pub(crate) struct DigestAdapter<'a> {
    url: Url,
    reader: Box<dyn Read + Send + 'a>,
    hashers: Vec<DownloadHasher>,
}

impl<'a> DigestAdapter<'a> {
    pub(crate) fn new(reader: Box<dyn Read + Send + 'a>, hashes: &Hashes, url: Url) -> Self {
        let hashers = DownloadHasher::all_supported(hashes);
        if !hashes.values.is_empty() && hashers.is_empty() {
            log::warn!(
                "None of the given hash algorithms at {} are supported",
                url.as_str()
            );
        }

        Self {
            url,
            reader,
            hashers,
        }
    }
}

impl<'a> Read for DigestAdapter<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = self.reader.read(buf)?;
        if size == 0 {
            for hasher in std::mem::take(&mut self.hashers) {
                hasher.validate(self.url.as_str())?;
            }
            Ok(size)
        } else {
            for hasher in &mut self.hashers {
                hasher.update(&buf[..size]);
            }

            Ok(size)
        }
    }
}

pub(crate) struct MaxSizeAdapter<'a> {
    reader: Box<dyn Read + Send + 'a>,
    /// How the `max_size` was specified. For example the max size of `root.json` is specified by
    /// the `max_root_size` argument in `Settings`. `specifier` is used to construct an error
    /// message when the `MaxSizeAdapter` detects that too many bytes have been read.
    specifier: &'static str,
    max_size: u64,
    counter: u64,
}

impl<'a> MaxSizeAdapter<'a> {
    pub(crate) fn new(
        reader: Box<dyn Read + Send + 'a>,
        specifier: &'static str,
        max_size: u64,
    ) -> Self {
        Self {
            reader,
            specifier,
            max_size,
            counter: 0,
        }
    }
}

impl<'a> Read for MaxSizeAdapter<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = self.reader.read(buf)?;
        self.counter += size as u64;
        if self.counter > self.max_size {
            error::MaxSizeExceededSnafu {
                max_size: self.max_size,
                specifier: self.specifier,
            }
            .fail()?;
        }
        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{DigestAdapter, MaxSizeAdapter};
    use crate::schema::Hashes;
    use std::collections::HashMap;
    use std::io::{Cursor, Read};
    use url::Url;

    #[test]
    fn test_max_size_adapter() {
        let mut reader = MaxSizeAdapter::new(Box::new(Cursor::new(b"hello".to_vec())), "test", 5);
        let mut buf = Vec::new();
        assert!(reader.read_to_end(&mut buf).is_ok());
        assert_eq!(buf, b"hello");

        let mut reader = MaxSizeAdapter::new(Box::new(Cursor::new(b"hello".to_vec())), "test", 4);
        let mut buf = Vec::new();
        assert!(reader.read_to_end(&mut buf).is_err());
    }

    #[test]
    fn test_digest_adapter() {
        let mut good_hashes = HashMap::new();
        good_hashes.insert(
            "sha256".to_string(),
            serde_json::Value::String(
                "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824".to_string(),
            ),
        );

        let mut reader = DigestAdapter::new(
            Box::new(Cursor::new(b"hello".to_vec())),
            &Hashes {
                values: good_hashes,
            },
            Url::parse("file:///").unwrap(),
        );
        let mut buf = Vec::new();
        assert!(reader.read_to_end(&mut buf).is_ok());
        assert_eq!(buf, b"hello");

        let mut bad_hashes = HashMap::new();
        bad_hashes.insert(
            "sha256".to_string(),
            serde_json::Value::String(
                "0ebdc3317b75839f643387d783535adc360ca01f33c75f7c1e7373adcd675c0b".to_string(),
            ),
        );

        let mut reader = DigestAdapter::new(
            Box::new(Cursor::new(b"hello".to_vec())),
            &Hashes { values: bad_hashes },
            Url::parse("file:///").unwrap(),
        );
        let mut buf = Vec::new();
        assert!(reader.read_to_end(&mut buf).is_err());
    }
}
