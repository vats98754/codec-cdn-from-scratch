use std::io::{Read, Write};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crate::error::{CodecError, Result};

/// Simple bitstream reader for parsing bitstreams
pub struct BitstreamReader<R: Read> {
    inner: R,
    bit_buffer: u64,
    bit_count: u8,
}

impl<R: Read> BitstreamReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            inner: reader,
            bit_buffer: 0,
            bit_count: 0,
        }
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        Ok(self.inner.read_u32::<LittleEndian>()?)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        Ok(self.inner.read_u16::<LittleEndian>()?)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        Ok(self.inner.read_u8()?)
    }

    pub fn read_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        self.inner.read_exact(buf)?;
        Ok(())
    }
}

/// Simple bitstream writer for generating bitstreams
pub struct BitstreamWriter<W: Write> {
    inner: W,
    bit_buffer: u64,
    bit_count: u8,
}

impl<W: Write> BitstreamWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            inner: writer,
            bit_buffer: 0,
            bit_count: 0,
        }
    }

    pub fn write_u32(&mut self, value: u32) -> Result<()> {
        Ok(self.inner.write_u32::<LittleEndian>(value)?)
    }

    pub fn write_u16(&mut self, value: u16) -> Result<()> {
        Ok(self.inner.write_u16::<LittleEndian>(value)?)
    }

    pub fn write_u8(&mut self, value: u8) -> Result<()> {
        Ok(self.inner.write_u8(value)?)
    }

    pub fn write_bytes(&mut self, buf: &[u8]) -> Result<()> {
        self.inner.write_all(buf)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.inner.flush()?;
        Ok(())
    }
}