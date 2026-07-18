//! Wire framing per docs/specs/15-NOVA-BUS-PROTOCOL-SPEC.md §2:
//! a 4-byte little-endian length prefix followed by a Protobuf-encoded
//! `Envelope`, capped at `MAX_MESSAGE_SIZE`.

use crate::proto::Envelope;
use prost::Message;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Per spec §2: bounds broker memory against a single oversized message.
pub const MAX_MESSAGE_SIZE: usize = 4 * 1024 * 1024;

pub async fn write_envelope<W: AsyncWrite + Unpin>(
    writer: &mut W,
    envelope: &Envelope,
) -> std::io::Result<()> {
    let mut buf = Vec::with_capacity(envelope.encoded_len());
    envelope
        .encode(&mut buf)
        .expect("Vec<u8> buffer never returns encode errors");
    if buf.len() > MAX_MESSAGE_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("envelope of {} bytes exceeds MAX_MESSAGE_SIZE", buf.len()),
        ));
    }
    writer.write_u32_le(buf.len() as u32).await?;
    writer.write_all(&buf).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn read_envelope<R: AsyncRead + Unpin>(
    reader: &mut R,
) -> std::io::Result<Option<Envelope>> {
    let len = match reader.read_u32_le().await {
        Ok(len) => len as usize,
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    };
    if len > MAX_MESSAGE_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("incoming envelope of {len} bytes exceeds MAX_MESSAGE_SIZE — protocol violation"),
        ));
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    let envelope = Envelope::decode(buf.as_slice())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(Some(envelope))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{envelope::Kind, Ping};

    #[tokio::test]
    async fn round_trips_an_envelope() {
        let (mut client, mut server) = tokio::io::duplex(1024);

        let sent = Envelope {
            protocol_version: 1,
            trace_id: None,
            kind: Some(Kind::Ping(Ping {})),
        };
        write_envelope(&mut client, &sent).await.unwrap();

        let received = read_envelope(&mut server).await.unwrap().unwrap();
        assert_eq!(received.protocol_version, 1);
        assert!(matches!(received.kind, Some(Kind::Ping(_))));
    }

    #[tokio::test]
    async fn read_on_closed_stream_returns_none() {
        let (client, mut server) = tokio::io::duplex(1024);
        drop(client);
        let result = read_envelope(&mut server).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn oversized_length_prefix_is_rejected() {
        let (mut client, mut server) = tokio::io::duplex(8 * 1024 * 1024);
        client
            .write_u32_le((MAX_MESSAGE_SIZE + 1) as u32)
            .await
            .unwrap();
        let result = read_envelope(&mut server).await;
        assert!(result.is_err());
    }
}
