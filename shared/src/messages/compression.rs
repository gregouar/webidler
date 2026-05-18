use std::{
    borrow::Cow,
    io::{Read, Write},
};

use anyhow::Result;
use flate2::{Compression, read::DeflateDecoder, write::DeflateEncoder};

const MAGIC: &[u8; 4] = b"WIC1";
const RAW: u8 = 0;
const DEFLATE: u8 = 1;

const MIN_COMPRESSION_SIZE: usize = 1024;
const MAX_DECOMPRESSED_SIZE: usize = 256 * 1024;

pub fn encode_payload(raw: Vec<u8>) -> Result<Vec<u8>> {
    if raw.len() < MIN_COMPRESSION_SIZE {
        return Ok(raw);
    }

    let mut encoded = Vec::with_capacity(raw.len() / 2);
    encoded.extend_from_slice(MAGIC);
    encoded.push(DEFLATE);

    let mut encoder = DeflateEncoder::new(encoded, Compression::fast());
    encoder.write_all(&raw)?;
    let encoded = encoder.finish()?;

    if encoded.len() >= raw.len() {
        return Ok(raw);
    }

    Ok(encoded)
}

pub fn decode_payload(payload: &[u8]) -> Result<Cow<'_, [u8]>> {
    let Some(rest) = payload.strip_prefix(MAGIC) else {
        return Ok(Cow::Borrowed(payload));
    };

    let Some((&encoding, encoded)) = rest.split_first() else {
        anyhow::bail!("compressed websocket payload missing encoding byte");
    };

    match encoding {
        RAW => Ok(Cow::Borrowed(encoded)),
        DEFLATE => {
            let decoder = DeflateDecoder::new(encoded);
            let mut limited = decoder.take((MAX_DECOMPRESSED_SIZE + 1) as u64);
            let mut decoded = Vec::new();
            limited.read_to_end(&mut decoded)?;

            if decoded.len() > MAX_DECOMPRESSED_SIZE {
                anyhow::bail!("compressed websocket payload exceeds decompressed size limit");
            }

            Ok(Cow::Owned(decoded))
        }
        _ => anyhow::bail!("unknown websocket payload encoding: {encoding}"),
    }
}
