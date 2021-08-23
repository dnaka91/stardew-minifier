#![deny(rust_2018_idioms, clippy::all, clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

use std::{borrow::Cow, str::FromStr};

use anyhow::{bail, Context, Result};
use camino::Utf8PathBuf;
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use models::ModData;
use structopt::{clap::AppSettings, StructOpt};

mod archive;
mod extract;
mod minify;
mod models;

#[derive(StructOpt)]
#[structopt(
    about,
    author,
    setting = AppSettings::ColoredHelp,
    setting = AppSettings::ArgRequiredElseHelp,
)]
pub struct Opt {
    /// Don't shrink JSON files.
    #[structopt(long)]
    no_json: bool,
    /// Don't shrink image (png) files.
    #[structopt(long)]
    no_images: bool,
    /// Don't shrink tile (tmx/tsx) files.
    #[structopt(long)]
    no_tiles: bool,
    /// The archive format to use.
    #[structopt(long, default_value ="zstd", possible_values = &["zstd", "zip"])]
    format: Format,
    /// Path to either a mod archive file (*.zip, *.tzst or *.tar.zst) or a folder containing the
    /// mod's content.
    path: Utf8PathBuf,
}

enum Format {
    Zstd,
    Zip,
}

impl FromStr for Format {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.eq_ignore_ascii_case("zstd") {
            Self::Zstd
        } else if s.eq_ignore_ascii_case("zip") {
            Self::Zip
        } else {
            bail!("unsupported archive format `{}`", s)
        })
    }
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let data = extract::extract(&opt.path)?;
    minify::minify(&data, &opt)?;
    archive::archive(&data, &opt)?;
    cleanup(data)?;

    Ok(())
}

fn cleanup(data: ModData) -> Result<()> {
    let pb = create_spinner("[4/4] cleaning up", "[4/4] cleaned up");
    data.delete().context("failed cleaning up temp data")?;
    pb.finish_using_style();

    Ok(())
}

#[allow(clippy::non_ascii_literal)]
fn create_bar(
    len: usize,
    message: &'static str,
    finish_message: impl Into<Cow<'static, str>>,
) -> ProgressBar {
    const TEMPLATE: &str =
        "{spinner:.green} {msg:25} |{bar:40.cyan/blue}| {pos:>6} / {len} ({eta})";

    let pb = ProgressBar::new(len as u64);
    pb.set_message(message);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(TEMPLATE)
            .progress_chars("█▉▊▋▌▍▎▏  ")
            .on_finish(ProgressFinish::WithMessage(finish_message.into())),
    );
    pb.enable_steady_tick(250);
    pb
}

fn create_spinner(
    message: &'static str,
    finish_message: impl Into<Cow<'static, str>>,
) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_message(message);
    pb.set_style(
        ProgressStyle::default_spinner()
            .on_finish(ProgressFinish::WithMessage(finish_message.into())),
    );
    pb.enable_steady_tick(250);
    pb
}
