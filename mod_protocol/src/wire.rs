use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt;
#[cfg(feature = "framing")]
use std::io::{Read, Write};

pub mod limits
{
    pub const CLIENTBOUND_MAX: usize = 1 << 20;
    pub const SERVERBOUND_MAX: usize = 32 * 1024;
    pub const SERVERBOUND_BUDGET_PER_TICK: usize = 64 * 1024;
    pub const COMPRESS_THRESHOLD: usize = 512;
}

/// the top bit of the channel varint marks a deflated body
pub const COMPRESSED_FLAG: u32 = 1 << 31;
pub const CONTROL_CHANNEL: u32 = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError
{
    Truncated,
    VarintTooLong,
    TooLarge(usize, usize),
    Codec(String),
    Inflate(String),
}

impl fmt::Display for WireError
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            WireError::Truncated => write!(formatter, "payload ended early"),
            WireError::VarintTooLong => write!(formatter, "varint longer than 5 bytes"),
            WireError::TooLarge(size, limit) => write!(formatter, "payload of {size} bytes exceeds {limit}"),
            WireError::Codec(reason) => write!(formatter, "payload codec: {reason}"),
            WireError::Inflate(reason) => write!(formatter, "payload inflate: {reason}"),
        }
    }
}

impl std::error::Error for WireError {}

pub fn encode<T: Serialize>(value: &T) -> Vec<u8>
{
    postcard::to_allocvec(value).expect("postcard encodes every protocol type into a vec")
}

pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WireError>
{
    postcard::from_bytes(bytes).map_err(|error| WireError::Codec(error.to_string()))
}

/// the named form keeps field names, so a type sent this way may gain defaulted fields and still read old bytes
pub fn encodeNamed<T: Serialize>(value: &T) -> Vec<u8>
{
    rmp_serde::to_vec_named(value).expect("every protocol type has a messagepack form")
}

pub fn decodeNamed<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WireError>
{
    rmp_serde::from_slice(bytes).map_err(|error| WireError::Codec(error.to_string()))
}

/// first byte of data a mod stores in the host since abi 4, the named form follows it
pub const STORED_MARK: u8 = 0xB4;

/// what a mod keeps on a tile, an entity, a player, a stack or a world, written so its struct can change shape later
pub fn encodeStored<T: Serialize>(value: &T) -> Vec<u8>
{
    let mut bytes: Vec<u8> = vec![STORED_MARK];
    bytes.extend_from_slice(&encodeNamed(value));
    bytes
}

/// data without the mark was stored before abi 4 in the positional form and is read that way
pub fn decodeStored<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WireError>
{
    match bytes.split_first()
    {
        Some((mark, named)) if *mark == STORED_MARK => decodeNamed(named).or_else(|_| decode(bytes)),
        _ => decode(bytes),
    }
}

/// for `#[serde(with = "crate::wire::named")]`: a field carried as named bytes inside a positional message
pub mod named
{
    use serde::de::{DeserializeOwned, Error as _};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<T: Serialize, S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    {
        serializer.serialize_bytes(&super::encodeNamed(value))
    }

    pub fn deserialize<'de, T: DeserializeOwned, D: Deserializer<'de>>(deserializer: D) -> Result<T, D::Error>
    {
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        super::decodeNamed(&bytes).map_err(D::Error::custom)
    }
}

mod varint
{
    pub const PAYLOAD: u32 = 0x7F;
    pub const MORE: u8 = 0x80;
    pub const BITS: u32 = 7;
    pub const MAX_BYTES: u32 = 5;
}

pub fn writeVarint(out: &mut Vec<u8>, mut value: u32)
{
    loop
    {
        let byte: u8 = (value & varint::PAYLOAD) as u8;

        value >>= varint::BITS;
        if value == 0
        {
            out.push(byte);
            return;
        }

        out.push(byte | varint::MORE);
    }
}

pub fn readVarint(bytes: &[u8], position: &mut usize) -> Result<u32, WireError>
{
    let mut value: u32 = 0;

    for shift in 0..varint::MAX_BYTES
    {
        let byte: u8 = *bytes.get(*position).ok_or(WireError::Truncated)?;

        *position += 1;
        value |= (byte as u32 & varint::PAYLOAD) << (shift * varint::BITS);

        if byte & varint::MORE == 0
        {
            return Ok(value);
        }
    }

    Err(WireError::VarintTooLong)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame
{
    pub m_channel: u32,
    pub m_data: Vec<u8>,
}

/// bodies over the threshold are deflated and flagged in the channel's top bit
#[cfg(feature = "framing")]
pub fn compress(channel: u32, data: &[u8]) -> (u32, Vec<u8>)
{
    if data.len() <= limits::COMPRESS_THRESHOLD
    {
        return (channel, data.to_vec());
    }

    let mut encoder: flate2::write::ZlibEncoder<Vec<u8>> =
        flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(data).expect("writing into a vec cannot fail");
    (channel | COMPRESSED_FLAG, encoder.finish().expect("finishing into a vec cannot fail"))
}

/// limit bounds both the wire body and the inflated body
#[cfg(feature = "framing")]
pub fn decompress(channel: u32, body: &[u8], limit: usize) -> Result<Frame, WireError>
{
    if body.len() > limit
    {
        return Err(WireError::TooLarge(body.len(), limit));
    }
    if channel & COMPRESSED_FLAG == 0
    {
        return Ok(Frame { m_channel: channel, m_data: body.to_vec() });
    }
    let mut inflated: Vec<u8> = Vec::new();
    let read: std::io::Result<usize> =
        flate2::read::ZlibDecoder::new(body).take(limit as u64 + 1).read_to_end(&mut inflated);
    read.map_err(|error| WireError::Inflate(error.to_string()))?;

    if inflated.len() > limit
    {
        return Err(WireError::TooLarge(inflated.len(), limit));
    }

    Ok(Frame { m_channel: channel & !COMPRESSED_FLAG, m_data: inflated })
}

#[cfg(feature = "framing")]
pub fn encodeFrame(channel: u32, data: &[u8]) -> Vec<u8>
{
    let (flagged, body): (u32, Vec<u8>) = compress(channel, data);
    let mut out: Vec<u8> = Vec::with_capacity(body.len() + 2 * varint::MAX_BYTES as usize);
    writeVarint(&mut out, flagged);
    writeVarint(&mut out, body.len() as u32);
    out.extend_from_slice(&body);
    out
}

#[cfg(feature = "framing")]
pub fn decodeFrame(bytes: &[u8], limit: usize) -> Result<Frame, WireError>
{
    let mut position: usize = 0;
    let channel: u32 = readVarint(bytes, &mut position)?;
    let length: usize = readVarint(bytes, &mut position)? as usize;
    if length > limit
    {
        return Err(WireError::TooLarge(length, limit));
    }

    let body: &[u8] = bytes.get(position..position + length).ok_or(WireError::Truncated)?;
    decompress(channel, body, limit)
}

#[cfg(all(test, feature = "framing"))]
mod tests
{
    use super::*;

    #[test]
    fn framesRoundTrip()
    {
        let small: Vec<u8> = vec![1, 2, 3];
        let big: Vec<u8> = vec![7; 4000];
        for data in [small, big]
        {
            let frame: Vec<u8> = encodeFrame(5, &data);
            let decoded: Frame = decodeFrame(&frame, limits::CLIENTBOUND_MAX).expect("decodes");
            assert_eq!(decoded.m_channel, 5);
            assert_eq!(decoded.m_data, data);
        }
    }

    #[test]
    fn rejectsOversizeInflate()
    {
        let frame: Vec<u8> = encodeFrame(1, &vec![0; 100000]);
        assert!(matches!(decodeFrame(&frame, 1000), Err(WireError::TooLarge(_, _))));
    }
}
