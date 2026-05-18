use std::borrow::Cow;

#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;

#[cfg(target_arch = "wasm32")]
use ruzstd::io::Read;

use anyhow::Result;

const MAGIC: &[u8; 4] = b"WIC2";
const ZSTD: u8 = 1;

const MIN_COMPRESSION_SIZE: usize = 1024;
const MAX_DECOMPRESSED_SIZE: usize = 256 * 1024;

pub fn encode_payload(raw: Vec<u8>) -> Result<Vec<u8>> {
    if let Some(encoded) = encode_payload_from_slice(&raw)? {
        return Ok(encoded);
    }

    Ok(raw)
}

pub fn encode_payload_from_slice(raw: &[u8]) -> Result<Option<Vec<u8>>> {
    if raw.len() < MIN_COMPRESSION_SIZE {
        return Ok(None);
    }

    encode_payload_inner(raw)
}

#[cfg(not(target_arch = "wasm32"))]
fn encode_payload_inner(raw: &[u8]) -> Result<Option<Vec<u8>>> {
    let mut encoded = Vec::with_capacity(raw.len() / 2);
    encoded.extend_from_slice(MAGIC);
    encoded.push(ZSTD);

    let mut encoder = zstd::stream::write::Encoder::new(encoded, 1)?;
    encoder.write_all(&raw)?;
    let encoded = encoder.finish()?;

    if encoded.len() >= raw.len() {
        return Ok(None);
    }

    Ok(Some(encoded))
}

#[cfg(target_arch = "wasm32")]
fn encode_payload_inner(_raw: &[u8]) -> Result<Option<Vec<u8>>> {
    Ok(None)
}

pub fn decode_payload(payload: &[u8]) -> Result<Cow<'_, [u8]>> {
    let Some(rest) = payload.strip_prefix(MAGIC) else {
        return Ok(Cow::Borrowed(payload));
    };

    let Some((&encoding, encoded)) = rest.split_first() else {
        anyhow::bail!("compressed websocket payload missing encoding byte");
    };

    match encoding {
        ZSTD => decode_zstd(encoded).map(Cow::Owned),
        _ => anyhow::bail!("unknown websocket payload encoding: {encoding}"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn decode_zstd(encoded: &[u8]) -> Result<Vec<u8>> {
    Ok(zstd::bulk::decompress(encoded, MAX_DECOMPRESSED_SIZE)?)
}

#[cfg(target_arch = "wasm32")]
fn decode_zstd(mut encoded: &[u8]) -> Result<Vec<u8>> {
    let decoder = ruzstd::decoding::StreamingDecoder::new(&mut encoded)
        .map_err(|e| anyhow::format_err!("failed to create zstd decoder: {e:?}"))?;
    let mut limited = decoder.take((MAX_DECOMPRESSED_SIZE + 1) as u64);
    let mut decoded = Vec::new();

    limited
        .read_to_end(&mut decoded)
        .map_err(|e| anyhow::format_err!("failed to decode zstd payload: {e:?}"))?;

    if decoded.len() > MAX_DECOMPRESSED_SIZE {
        anyhow::bail!("compressed websocket payload exceeds decompressed size limit");
    }

    Ok(decoded)
}
