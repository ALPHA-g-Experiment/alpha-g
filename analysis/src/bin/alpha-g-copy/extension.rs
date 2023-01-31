use anyhow::{Context, Result};
use clap::ValueEnum;
use lz4::Decoder;
use std::fmt;
use std::fs::File;
use std::path::Path;

/// Extension i.e. compression of MIDAS files
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Extension {
    Lz4,
}
impl fmt::Display for Extension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Extension::Lz4 => write!(f, ".lz4"),
        }
    }
}

/// Decompress `.lz4` file
pub fn decompress_lz4(source: &Path, destination: &Path) -> Result<()> {
    let input_file =
        File::open(source).with_context(|| format!("failed to open `{}`", source.display()))?;
    let mut decoder = Decoder::new(input_file).context("failed to create lz4 encoder")?;
    let mut output_file = File::create(destination).with_context(|| {
        format!(
            "failed to create output file at `{}`",
            destination.display()
        )
    })?;
    std::io::copy(&mut decoder, &mut output_file).with_context(|| {
        format!(
            "failed to write decompressed data to `{}`",
            destination.display()
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests;
