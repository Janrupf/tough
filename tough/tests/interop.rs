// Copyright 2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

use reqwest::Url;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tough::Repository;

fn test_data() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
}

fn dir_url<P: AsRef<Path>>(path: P) -> String {
    Url::from_directory_path(path).unwrap().to_string()
}

fn read_to_end<R: Read>(mut reader: R) -> Vec<u8> {
    let mut v = Vec::new();
    reader.read_to_end(&mut v).unwrap();
    v
}

/// Test that `tough` can process repositories generated by [`tuf`], the reference Python
/// implementation.
///
/// [`tuf`]: https://github.com/theupdateframework/tuf
#[test]
fn test_tuf_reference_impl() {
    let base = test_data().join("tuf-reference-impl");
    let datastore = TempDir::new().unwrap();

    let repo = Repository::load(
        File::open(base.join("metadata").join("1.root.json")).unwrap(),
        &datastore,
        1024 * 1024,
        1024 * 1024,
        1024 * 1024,
        &dir_url(base.join("metadata")),
        &dir_url(base.join("targets")),
    )
    .unwrap();

    assert_eq!(
        read_to_end(repo.read_target("file1.txt").unwrap().unwrap()),
        &b"This is an example target file."[..]
    );
    assert_eq!(
        read_to_end(repo.read_target("file2.txt").unwrap().unwrap()),
        &b"This is an another example target file."[..]
    );
    assert_eq!(
        repo.targets()
            .get("file1.txt")
            .unwrap()
            .custom
            .get("file_permissions")
            .unwrap(),
        "0644"
    );
}
