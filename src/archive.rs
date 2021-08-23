use std::{
    fs::File,
    io::{BufWriter, Write},
};

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use indicatif::ProgressIterator;
use tar::{Builder as TarBuilder, HeaderMode};
use zip::{write::FileOptions, ZipWriter};

use crate::{create_bar, models::ModData, Format, Opt};

pub fn archive(data: &ModData, opt: &Opt) -> Result<Utf8PathBuf> {
    let out_name = output_file_name(&opt.path);
    match opt.format {
        Format::Zstd => create_tar_zst_archive(data, out_name),
        Format::Zip => create_zip_archive(data, out_name),
    }
    .context("failed creating archive")
}

fn create_tar_zst_archive(data: &ModData, mut out: Utf8PathBuf) -> Result<Utf8PathBuf> {
    out.set_extension("tzst");

    let file = File::create(&out)?;
    let file = BufWriter::new(file);
    let file = {
        let mut enc = zstd::Encoder::new(file, 19)?;
        enc.include_checksum(true)?;
        enc.include_contentsize(true)?;
        enc
    };

    let mut builder = TarBuilder::new(file);
    let pb = create_bar(
        data.files().len(),
        "[3/4] creating archive",
        "[3/4] archive created",
    );

    builder.mode(HeaderMode::Deterministic);

    for file in data.files().iter().progress_with(pb) {
        builder.append_path_with_name(data.path().join(file), file)?;
    }

    builder.into_inner()?.finish()?.into_inner()?.flush()?;

    Ok(out)
}

fn create_zip_archive(data: &ModData, mut out: Utf8PathBuf) -> Result<Utf8PathBuf> {
    out.set_extension("zip");

    let file = File::create(&out)?;
    let file = BufWriter::new(file);
    let mut builder = ZipWriter::new(file);
    let pb = create_bar(
        data.files().len(),
        "[3/4] creating archive",
        "[3/4] archive created",
    );

    for file in data.files().iter().progress_with(pb) {
        builder
            .start_file(file.as_str(), FileOptions::default())
            .context("failed creating zip entry")?;

        let mut file = File::open(file).context("failed opening file")?;
        std::io::copy(&mut file, &mut builder).context("failed adding file to zip archive")?;
    }

    builder.finish()?.into_inner()?.flush()?;

    Ok(out)
}

fn output_file_name(path: &Utf8Path) -> Utf8PathBuf {
    let out = path.file_name().unwrap_or("file");
    let out = out
        .strip_suffix(".zip")
        .or_else(|| out.strip_suffix(".tzst"))
        .or_else(|| out.strip_suffix(".tar.zst"))
        .unwrap_or(out);
    let out = format!("{}.out", out);

    #[allow(clippy::map_unwrap_or)]
    path.parent()
        .map(|p| p.join(&out))
        .unwrap_or_else(|| out.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_file_name() {
        let cases = &[
            ("file.out", "/"),
            ("file.out", "file"),
            ("file.out", "file.zip"),
            ("file.out", "file.tzst"),
            ("file.out", "file.tar.zst"),
            ("file-1.0.0.out", "file-1.0.0.zip"),
            ("file-1.0.0.out", "file-1.0.0.tzst"),
            ("file-1.0.0.out", "file-1.0.0.tar.zst"),
            ("/temp/file-1.0.0.out", "/temp/file-1.0.0.zip"),
            ("/temp/file-1.0.0.out", "/temp/file-1.0.0.tzst"),
            ("/temp/file-1.0.0.out", "/temp/file-1.0.0.tar.zst"),
        ];

        for (expect, input) in cases {
            assert_eq!(*expect, output_file_name(Utf8Path::new(input)));
        }
    }
}
