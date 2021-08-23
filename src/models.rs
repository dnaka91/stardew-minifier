use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use tempfile::TempDir;

pub struct ModData {
    dir: TempDir,
    path: Utf8PathBuf,
    files: Vec<Utf8PathBuf>,
}

impl ModData {
    pub fn new(dir: TempDir, path: Utf8PathBuf, files: Vec<Utf8PathBuf>) -> Self {
        Self { dir, path, files }
    }

    pub fn path(&self) -> &Utf8Path {
        &self.path
    }

    pub fn files(&self) -> &[Utf8PathBuf] {
        &self.files
    }

    pub fn delete(self) -> Result<()> {
        self.dir.close().map_err(Into::into)
    }
}
