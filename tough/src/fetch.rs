// Copyright 2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::error::{self, Result};
use crate::io::{DigestAdapter, MaxSizeAdapter};
use crate::transport::Transport;
use snafu::ResultExt;
use std::io::Read;
use url::Url;
use crate::schema::Hashes;

pub(crate) fn fetch_max_size<'a>(
    transport: &'a dyn Transport,
    url: Url,
    max_size: u64,
    specifier: &'static str,
) -> Result<impl Read + Send + 'a> {
    Ok(MaxSizeAdapter::new(
        transport
            .fetch(url.clone())
            .context(error::TransportSnafu { url })?,
        specifier,
        max_size,
    ))
}

pub(crate) fn fetch_hashed<'a>(
    transport: &'a dyn Transport,
    url: Url,
    size: u64,
    specifier: &'static str,
    hashes: &Hashes,
) -> Result<impl Read + Send + 'a> {
    Ok(DigestAdapter::new(
        Box::new(MaxSizeAdapter::new(
            transport
                .fetch(url.clone())
                .context(error::TransportSnafu { url: url.clone() })?,
            specifier,
            size,
        )),
        hashes,
        url,
    ))
}
