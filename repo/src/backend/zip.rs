//! Minimal store-only ZIP writer (no compression, no dependencies).
//!
//! This implementation writes the ZIP format as documented in APPNOTE.TXT
//! from PKWARE. Each file entry uses method 0 (stored), which is acceptable
//! for text logs and small JSON snapshots. The writer produces a valid
//! archive that standard unzip utilities (and `zip -l`) can read.
//!
//! Not supported: compression, encryption, zip64, large (>4 GiB) archives.

use std::io::{Error, ErrorKind, Result as IoResult, Write};

pub struct ZipWriter<W: Write> {
    inner: W,
    offset: u64,
    entries: Vec<CentralEntry>,
}

struct CentralEntry {
    name: Vec<u8>,
    crc32: u32,
    size: u32,
    offset: u32,
    dostime: u16,
    dosdate: u16,
}

impl<W: Write> ZipWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner, offset: 0, entries: Vec::new() }
    }

    pub fn add_file(&mut self, name: &str, data: &[u8]) -> IoResult<()> {
        if data.len() > u32::MAX as usize {
            return Err(Error::new(ErrorKind::InvalidInput, "file too large for non-zip64"));
        }
        let crc = crc32(data);
        let size = data.len() as u32;
        let (dosdate, dostime) = dos_datetime_now();

        // Local file header
        // signature 0x04034b50
        self.write_u32(0x04034b50)?;
        self.write_u16(20)?;                   // version needed
        self.write_u16(0)?;                    // gp bit flag
        self.write_u16(0)?;                    // compression = stored
        self.write_u16(dostime)?;
        self.write_u16(dosdate)?;
        self.write_u32(crc)?;
        self.write_u32(size)?;                 // compressed size
        self.write_u32(size)?;                 // uncompressed size
        self.write_u16(name.len() as u16)?;
        self.write_u16(0)?;                    // extra field length
        self.write_all(name.as_bytes())?;
        let offset = (self.offset - (30 + name.len() as u64)) as u32;
        // Re-compute: offset was tracked after header+name; rewind logic
        // below recomputes from entries vector instead. Simpler: capture
        // BEFORE writing local header.
        let _ = offset;

        // Note: we corrected offset capture below by pushing BEFORE writing
        // file data. But since we've already written the header above, we
        // should have saved the offset first. Rewrite the flow: save start
        // offset, write header and data, record entry.
        // Since we've already written above, we compute start offset from
        // current pointer minus header (30) minus name.len().
        let start = self.offset - 30 - name.len() as u64;

        self.write_all(data)?;

        self.entries.push(CentralEntry {
            name: name.as_bytes().to_vec(),
            crc32: crc,
            size,
            offset: start as u32,
            dostime,
            dosdate,
        });
        Ok(())
    }

    pub fn finish(mut self) -> IoResult<W> {
        let cd_start = self.offset;
        // Take ownership of the entries vector so we can call the mutable
        // helpers (write_u32, write_u16, write_all) inside the loop without
        // conflicting with an outstanding immutable borrow of `self.entries`.
        let entries = std::mem::take(&mut self.entries);
        for e in &entries {
            // Central directory header
            self.write_u32(0x02014b50)?;
            self.write_u16(20)?;               // version made by
            self.write_u16(20)?;               // version needed
            self.write_u16(0)?;                // flags
            self.write_u16(0)?;                // method stored
            self.write_u16(e.dostime)?;
            self.write_u16(e.dosdate)?;
            self.write_u32(e.crc32)?;
            self.write_u32(e.size)?;
            self.write_u32(e.size)?;
            self.write_u16(e.name.len() as u16)?;
            self.write_u16(0)?;                // extra len
            self.write_u16(0)?;                // comment len
            self.write_u16(0)?;                // disk number
            self.write_u16(0)?;                // internal attrs
            self.write_u32(0)?;                // external attrs
            self.write_u32(e.offset)?;
            self.inner.write_all(&e.name)?;
            self.offset += e.name.len() as u64;
        }
        let entries_len = entries.len() as u16;
        let cd_end = self.offset;
        let cd_size = (cd_end - cd_start) as u32;

        // End of central directory
        self.write_u32(0x06054b50)?;
        self.write_u16(0)?;                    // disk number
        self.write_u16(0)?;                    // disk with CD
        self.write_u16(entries_len)?;          // entries on this disk
        self.write_u16(entries_len)?;          // total entries
        self.write_u32(cd_size)?;
        self.write_u32(cd_start as u32)?;
        self.write_u16(0)?;                    // comment length

        Ok(self.inner)
    }

    fn write_u16(&mut self, v: u16) -> IoResult<()> {
        self.inner.write_all(&v.to_le_bytes())?;
        self.offset += 2;
        Ok(())
    }
    fn write_u32(&mut self, v: u32) -> IoResult<()> {
        self.inner.write_all(&v.to_le_bytes())?;
        self.offset += 4;
        Ok(())
    }
    fn write_all(&mut self, data: &[u8]) -> IoResult<()> {
        self.inner.write_all(data)?;
        self.offset += data.len() as u64;
        Ok(())
    }
}

/// CRC-32 / IEEE polynomial 0xEDB88320.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            if crc & 1 != 0 { crc = (crc >> 1) ^ 0xEDB88320; }
            else { crc >>= 1; }
        }
    }
    !crc
}

/// Current time encoded as DOS date (year-1980<<9|month<<5|day) and
/// DOS time (hour<<11|minute<<5|second/2).
fn dos_datetime_now() -> (u16, u16) {
    let dt = crate::common::CivilDateTime::now();
    let dosdate = (((dt.year - 1980).max(0) as u16) << 9) | ((dt.month as u16) << 5) | (dt.day as u16);
    let dostime = ((dt.hour as u16) << 11) | ((dt.minute as u16) << 5) | ((dt.second / 2) as u16);
    (dosdate, dostime)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_archive_has_eocd_signature() {
        let buf: Vec<u8> = Vec::new();
        let w = ZipWriter::new(buf);
        let out = w.finish().unwrap();
        // last 22 bytes contain EOCD; first 4 of EOCD are 0x06054b50
        let idx = out.len() - 22;
        assert_eq!(&out[idx..idx + 4], &[0x50, 0x4b, 0x05, 0x06]);
    }

    #[test]
    fn single_file_archive_structure() {
        let buf: Vec<u8> = Vec::new();
        let mut w = ZipWriter::new(buf);
        w.add_file("hello.txt", b"hi there").unwrap();
        let out = w.finish().unwrap();
        // Local file header signature appears at offset 0
        assert_eq!(&out[0..4], &[0x50, 0x4b, 0x03, 0x04]);
        // Contents present
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("hi there"));
        assert!(s.contains("hello.txt"));
    }

    #[test]
    fn crc32_vectors() {
        assert_eq!(crc32(b""), 0);
        assert_eq!(crc32(b"123456789"), 0xCBF43926);
    }
}
