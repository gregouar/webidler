use std::borrow::Cow;

#[cfg(not(target_arch = "wasm32"))]
use std::io::{Read as _, Write};

#[cfg(target_arch = "wasm32")]
use ruzstd::io::Read;

use anyhow::Result;

const MAGIC: &[u8; 4] = b"WIC2";
const ZSTD: u8 = 1;
const ZSTD_DICTIONARY: u8 = 2;

const MAX_DECOMPRESSED_SIZE: usize = 256 * 1024;

pub fn encode_payload(raw: Vec<u8>) -> Result<Vec<u8>> {
    if let Some(encoded) = encode_payload_from_slice(&raw, None)? {
        return Ok(encoded);
    }

    Ok(raw)
}

pub fn encode_payload_with_dictionary(raw: Vec<u8>, dictionary: Option<&[u8]>) -> Result<Vec<u8>> {
    if let Some(encoded) = encode_payload_from_slice(&raw, dictionary)? {
        return Ok(encoded);
    }

    Ok(raw)
}

pub fn encode_payload_from_slice(raw: &[u8], dictionary: Option<&[u8]>) -> Result<Option<Vec<u8>>> {
    encode_payload_inner(raw, dictionary)
}

#[cfg(not(target_arch = "wasm32"))]
fn encode_payload_inner(raw: &[u8], dictionary: Option<&[u8]>) -> Result<Option<Vec<u8>>> {
    let mut encoded = Vec::with_capacity(raw.len() / 2);
    encoded.extend_from_slice(MAGIC);

    let encoded = if let Some(dictionary) = dictionary.filter(|dictionary| !dictionary.is_empty()) {
        encoded.push(ZSTD_DICTIONARY);
        let mut compressor = zstd::bulk::Compressor::with_dictionary(1, dictionary)?;
        let compressed = compressor.compress(raw)?;
        encoded.extend_from_slice(&compressed);
        encoded
    } else {
        encoded.push(ZSTD);
        let mut encoder = zstd::stream::write::Encoder::new(encoded, 1)?;
        encoder.write_all(raw)?;
        encoder.finish()?
    };

    Ok(Some(encoded))
}

#[cfg(target_arch = "wasm32")]
fn encode_payload_inner(raw: &[u8], _dictionary: Option<&[u8]>) -> Result<Option<Vec<u8>>> {
    let mut encoded = Vec::with_capacity(raw.len() / 2);
    encoded.extend_from_slice(MAGIC);
    encoded.push(ZSTD);
    encoded.extend_from_slice(&ruzstd::encoding::compress_to_vec(
        raw,
        ruzstd::encoding::CompressionLevel::Fastest,
    ));
    Ok(Some(encoded))
}

pub fn decode_payload(payload: &[u8]) -> Result<Cow<'_, [u8]>> {
    decode_payload_with_dictionary(payload, None)
}

pub fn decode_payload_with_dictionary<'a>(
    payload: &'a [u8],
    dictionary: Option<&[u8]>,
) -> Result<Cow<'a, [u8]>> {
    let Some(rest) = payload.strip_prefix(MAGIC) else {
        return Ok(Cow::Borrowed(payload));
    };

    let Some((&encoding, encoded)) = rest.split_first() else {
        anyhow::bail!("compressed websocket payload missing encoding byte");
    };

    match encoding {
        ZSTD => decode_zstd(encoded).map(Cow::Owned),
        ZSTD_DICTIONARY => {
            let Some(dictionary) = dictionary.filter(|dictionary| !dictionary.is_empty()) else {
                anyhow::bail!("compressed websocket payload requires a zstd dictionary");
            };
            decode_zstd_with_dictionary(encoded, dictionary).map(Cow::Owned)
        }
        _ => anyhow::bail!("unknown websocket payload encoding: {encoding}"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn decode_zstd(encoded: &[u8]) -> Result<Vec<u8>> {
    Ok(zstd::bulk::decompress(encoded, MAX_DECOMPRESSED_SIZE)?)
}

#[cfg(not(target_arch = "wasm32"))]
fn decode_zstd_with_dictionary(encoded: &[u8], dictionary: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = zstd::stream::read::Decoder::with_dictionary(encoded, dictionary)?;
    let mut decoded = Vec::new();
    decoder
        .by_ref()
        .take((MAX_DECOMPRESSED_SIZE + 1) as u64)
        .read_to_end(&mut decoded)?;

    if decoded.len() > MAX_DECOMPRESSED_SIZE {
        anyhow::bail!("compressed websocket payload exceeds decompressed size limit");
    }

    Ok(decoded)
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

#[cfg(target_arch = "wasm32")]
fn decode_zstd_with_dictionary(mut encoded: &[u8], dictionary: &[u8]) -> Result<Vec<u8>> {
    let dictionary = ruzstd::decoding::Dictionary::decode_dict(dictionary)
        .map_err(|e| anyhow::format_err!("failed to decode zstd dictionary: {e:?}"))?;
    let mut frame_decoder = ruzstd::decoding::FrameDecoder::new();
    frame_decoder
        .add_dict(dictionary)
        .map_err(|e| anyhow::format_err!("failed to register zstd dictionary: {e:?}"))?;

    let decoder = ruzstd::decoding::StreamingDecoder::new_with_decoder(&mut encoded, frame_decoder)
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
