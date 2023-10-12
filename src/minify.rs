use std::{
    fs::File,
    io::{BufWriter, Cursor, ErrorKind, Write},
};

use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use encoding_rs::Encoding;
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{create_bar, models::ModData, Opt};

pub fn minify(data: &ModData, opt: &Opt) -> Result<()> {
    let pb = create_bar(
        data.files().len(),
        "[2/4] minifying files",
        "[2/4] files minified",
    );

    data.files()
        .par_iter()
        .progress_with(pb)
        .try_for_each(|file| {
            if let Some(ext) = file.extension() {
                match ext {
                    "json" if !opt.no_json => {
                        minify_json(data.path().join(file))
                            .with_context(|| format!("failed minifying json file {file:?}"))?;
                    }
                    "png" if !opt.no_images => {
                        minify_png(data.path().join(file))
                            .with_context(|| format!("failed minifying png file {file:?}"))?;
                    }
                    "tmx" | "tsx" if !opt.no_tiles => {
                        minify_xml(data.path().join(file))
                            .with_context(|| format!("failed minifying tmx/tsx file {file:?}"))?;
                    }
                    _ => {}
                }
            }

            Ok::<_, anyhow::Error>(())
        })
        .context("failed minifying files")
}

fn minify_json(path: Utf8PathBuf) -> Result<()> {
    let json = match std::fs::read_to_string(&path) {
        Ok(json) => json,
        Err(e) if e.kind() == ErrorKind::InvalidData => {
            if let Some(encoding) = get_encoding(&path) {
                let buf = std::fs::read(&path)?;
                let (json, _, _) = encoding.decode(&buf);
                json.into_owned()
            } else {
                bail!("unsupported encoding for file {:?}", path);
            }
        }
        Err(e) => return Err(e.into()),
    };
    let json = json5::from_str::<serde_json::Value>(&json).context("failed parsing json")?;

    let file = File::create(path)?;
    let mut file = BufWriter::new(file);

    serde_json::to_writer(&mut file, &json)?;

    file.into_inner()?.flush()?;

    Ok(())
}

fn minify_png(path: Utf8PathBuf) -> Result<()> {
    use oxipng::{Deflaters, InFile, Options, OutFile, StripChunks};

    let mut opts = Options::max_compression();
    opts.strip = StripChunks::All;
    opts.deflate = Deflaters::Libdeflater { compression: 12 };

    oxipng::optimize(
        &InFile::Path(path.into_std_path_buf()),
        &OutFile::Path {
            path: None,
            preserve_attrs: true,
        },
        &opts,
    )
    .map_err(Into::into)
}

fn minify_xml(path: Utf8PathBuf) -> Result<()> {
    use quick_xml::{events::Event, Reader, Writer};

    let mut reader = Reader::from_file(&path).context("failed opening file")?;
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    reader.trim_text(true);

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(event) => writer.write_event(event)?,
            Err(e) => return Err(e.into()),
        }
        buf.clear();
    }

    std::fs::write(path, writer.into_inner().into_inner())?;

    Ok(())
}

fn get_encoding(path: &Utf8Path) -> Option<&'static Encoding> {
    if path.parent().and_then(camino::Utf8Path::file_name) != Some("i18n")
        || path.extension() != Some("json")
    {
        return None;
    }

    Some(match path.file_stem().unwrap_or_default() {
        "ja" => encoding_rs::SHIFT_JIS,
        "zh" | "ko" => encoding_rs::UTF_8,
        "hu" => encoding_rs::WINDOWS_1250,
        "ru" => encoding_rs::WINDOWS_1251,
        "de" | "es" | "fr" | "it" | "pt" => encoding_rs::WINDOWS_1252,
        "tr" => encoding_rs::WINDOWS_1254,
        _ => return None,
    })
}
