use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind, Read, Write};
use std::path::Path;

use crate::errors::CaptureError;
use crate::services::CaptureCodec;
use crate::structs::{CaptureEvent, CaptureHeader};

// reads a .lmcap file event by event; a file cut short by a crash ends at its last whole record
pub struct CaptureReader {
    input: Box<dyn Read>,
    header: CaptureHeader,
    payload: Vec<u8>,
}

impl CaptureReader {
    pub fn open(path: &Path) -> Result<Self, CaptureError> {
        let mut file = BufReader::new(File::open(path)?);
        let (header, compression) = CaptureCodec::read_header(&mut file)?;
        let input: Box<dyn Read> = if compression == CaptureCodec::ZSTD {
            Box::new(zstd::Decoder::with_buffer(file)?)
        } else {
            Box::new(file)
        };
        Ok(Self { input, header, payload: Vec::new() })
    }

    pub fn header(&self) -> &CaptureHeader {
        &self.header
    }

    pub fn next_event(&mut self) -> Result<Option<CaptureEvent>, CaptureError> {
        let mut head = [0u8; CaptureCodec::RECORD_HEADER_BYTES];
        if !Self::read_whole(&mut self.input, &mut head)? {
            return Ok(None);
        }
        let len = u32::from_le_bytes([head[1], head[2], head[3], head[4]]) as usize;
        let expected = CaptureCodec::payload_bytes(head[0]).ok_or(CaptureError::UnknownTag(head[0]))?;
        if len != expected {
            return Err(CaptureError::WrongSize { tag: head[0], expected, actual: len });
        }
        self.payload.resize(len, 0);
        if !Self::read_whole(&mut self.input, &mut self.payload)? {
            return Ok(None);
        }
        CaptureCodec::decode(head[0], &self.payload).map(Some)
    }

    // rewrites the capture without compression, for readers that have no zstd
    pub fn unpack_to(mut self, path: &Path) -> Result<u64, CaptureError> {
        let mut out = BufWriter::new(File::create(path)?);
        CaptureCodec::write_header(&mut out, &self.header, CaptureCodec::RAW)?;
        let mut scratch = Vec::with_capacity(4096);
        let mut count = 0u64;
        while let Some(event) = self.next_event()? {
            scratch.clear();
            CaptureCodec::encode(&event, &mut scratch);
            out.write_all(&scratch)?;
            count += 1;
        }
        out.flush()?;
        Ok(count)
    }

    // false at a clean or torn end of the stream
    fn read_whole(input: &mut impl Read, buf: &mut [u8]) -> Result<bool, CaptureError> {
        match input.read_exact(buf) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == ErrorKind::UnexpectedEof => Ok(false),
            Err(error) => Err(error.into()),
        }
    }
}
