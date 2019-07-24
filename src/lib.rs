#![deny(unsafe_code)]

pub mod deflate;
pub mod error;
pub mod gzip;
pub mod parse;

#[cfg(test)]
mod tests {
    #[test]
    fn read_gzip() -> Result<(), Box<dyn std::error::Error>> {
        use std::{fs, path::PathBuf};

        let gzips_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("test-gzips");

        let gz_bytes = fs::read(gzips_dir.join("hello.txt.gz"))?;
        let ref_bytes = fs::read(gzips_dir.join("hello.txt"))?;
        let dec_bytes = super::gzip::Reader::read(&gz_bytes)?;

        assert_eq!(ref_bytes, dec_bytes);

        Ok(())
    }
}
