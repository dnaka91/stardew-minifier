use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter},
};

use anyhow::{bail, Context, Result};
use camino::Utf8Path;
use ignore::Walk;
use indicatif::ProgressIterator;
use tar::Archive as TarArchive;
use tempfile::TempDir;
use zip::ZipArchive;

use crate::{create_bar, create_spinner, ModData};

#[allow(clippy::case_sensitive_file_extension_comparisons)]
pub fn extract(path: &Utf8Path) -> Result<ModData> {
    Ok(if path.is_file() {
        let name = path.file_name().unwrap_or_default();

        if name.ends_with(".zip") {
            extract_zip(path).context("failed extracting zip archive")?
        } else if name.ends_with(".tzst") || name.ends_with(".tar.zst") {
            extract_tar_zst(path).context("failed extracting tar.zst archive")?
        } else {
            bail!("unsupported file type");
        }
    } else if path.is_dir() {
        copy_dir(path).context("failed copying directory")?
    } else {
        bail!("unsupported file or directory");
    })
}

fn extract_zip(path: &Utf8Path) -> Result<ModData> {
    let file = File::open(path).context("failed opening zip file")?;
    let file = BufReader::new(file);
    let mut zip = ZipArchive::new(file).context("failed reading zip file")?;
    let dir = TempDir::new().context("failed creating temp dir")?;
    let path = Utf8Path::from_path(dir.path())
        .context("not a valid UTF-8 path")?
        .to_owned();

    let mut files = Vec::with_capacity(zip.len());
    let pb = create_bar(zip.len(), "[1/4] extracting data", "[1/4] data extracted");

    for i in (0..zip.len()).progress_with(pb) {
        let mut entry = zip
            .by_index(i)
            .context("failed getting zip entry information")?;

        if !entry.is_file() {
            continue;
        }

        if let Some(name) = entry
            .enclosed_name()
            .map(|name| Utf8Path::from_path(name).map(ToOwned::to_owned))
        {
            let name = name.context("not a valid UTF-8 path")?;

            if let Some(parent) = name.parent() {
                fs::create_dir_all(path.join(parent))
                    .context("failed creating entry's parent folder(s)")?;
            }

            let out = File::create(path.join(&name))
                .context("failed creating output file for zip entry")?;
            let mut out = BufWriter::new(out);
            io::copy(&mut entry, &mut out).context("failed extracting zip entry")?;

            out.into_inner()?;

            files.push(name);
        }
    }

    Ok(ModData::new(dir, path, files))
}

fn extract_tar_zst(path: &Utf8Path) -> Result<ModData> {
    let file = File::open(path).context("failed opening tar.zst file")?;
    let file = BufReader::new(file);
    let file = zstd::Decoder::new(file).context("failed reading zstd data")?;
    let mut tar = TarArchive::new(file);
    let dir = TempDir::new().context("failed creating temp dir")?;
    let path = Utf8Path::from_path(dir.path())
        .context("not a valid UTF-8 path")?
        .to_owned();

    let mut files = Vec::new();
    let pb = create_spinner("[1/4] extracting data", "[1/4] data extracted");

    for entry in tar.entries()? {
        let mut entry = entry.context("failed getting tar entry")?;

        if !entry.header().entry_type().is_file() {
            continue;
        }

        if let Ok(name) = entry
            .path()
            .map(|path| Utf8Path::from_path(&path).map(ToOwned::to_owned))
        {
            let name = name.context("not a valid UTF-8 path")?;

            if let Some(parent) = name.parent() {
                fs::create_dir_all(dir.path().join(parent))
                    .context("failed creating entry's parent folder(s)")?;
            }

            let out = File::create(dir.path().join(&name))
                .context("failed creating output file for zip entry")?;
            let mut out = BufWriter::new(out);
            io::copy(&mut entry, &mut out).context("failed extracting zip entry")?;

            out.into_inner()?;

            files.push(name);
        }
    }

    pb.finish_using_style();

    Ok(ModData::new(dir, path, files))
}

fn copy_dir(source: &Utf8Path) -> Result<ModData> {
    let dir = TempDir::new().context("failed creating temp dir")?;
    let path = Utf8Path::from_path(dir.path())
        .context("not a valid UTF-8 path")?
        .to_owned();

    let mut files = Vec::new();
    let pb = create_spinner("[1/4] copying folder", "[1/4] folder copied");

    for entry in Walk::new(source) {
        let entry = entry.context("failed traversing folder")?;
        let ty = entry.file_type().unwrap();
        let entry_path = Utf8Path::from_path(entry.path()).context("not a valid UTF-8 path")?;

        if ty.is_dir() {
            fs::create_dir(path.join(entry_path)).context("failed creating folder")?;
        }

        if ty.is_file() {
            let mut file = File::open(entry_path).context("failed opening file")?;
            let mut out = File::create(path.join(entry_path))
                .context("failed creating output file for entry")?;

            io::copy(&mut file, &mut out).context("failed copying file")?;
            files.push(entry_path.to_owned());
        }
    }

    pb.finish_using_style();

    Ok(ModData::new(dir, path, files))
}
