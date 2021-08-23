# 🤖 Stardew Minifier

[![Build Status][build-img]][build-url]

[build-img]: https://img.shields.io/github/workflow/status/dnaka91/stardew-minifier/CI/main?style=for-the-badge
[build-url]: https://github.com/dnaka91/stardew-minifier/actions?query=workflow%3ACI

Shrink **Stardew Valley** mod files by optimizing assets.

## Build

To build this project have `rust` and `cargo` available in the latest version and run `cargo build`.
Now you will find the binary at `target/debug/stardew-minifier` which you can directly execute or
use `cargo run` for convenience.

## Usage

Call `stardew-minifier` with the path to a mod archive (usually a `*.zip` file, but `*.tzst` or
`*.tar.zst` are supported too) and it will extract the files to a temp folder, process each and pack
it back together into an archive.

Alternatively a folder to an already extracted mod can be passed as well.

The output archive is in `*.tzst` format by default, which is a **Zstandard** compressed **TAR**
archive. If that's problematic due to upload requirements or missing support for that format, the
`--format` flag can be used to change to a `*zip` archive instead.

Optionally specific types of optimizations can be disabled with any of the `--no-*` flags in case
it is not wished or the file type can't be processed properly. Especially JSON files are often not
proper in many mods.

For any further options run `stardew-minifier --help`.

### Sample output

```sh
$ stardew-minifier Better\ Artisan\ Good\ Icons-2080-1-5-0-1551768141.tzst
  [1/4] data extracted
  [2/4] files minified      |████████████████████████████████████████|     10 / 10 (0s)
  [3/4] archive created     |████████████████████████████████████████|     10 / 10 (0s)
  [4/4] cleaned up
```

## License

This project is licensed under the [AGPL-3.0 License](LICENSE) (or
<https://www.gnu.org/licenses/agpl-3.0.html>).
