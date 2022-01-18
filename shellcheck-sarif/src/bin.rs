#![doc(html_root_url = "https://docs.rs/shellcheck-sarif/0.2.25")]

//! This crate provides a command line tool to convert `shellcheck` diagnostic
//! output into SARIF.
//!
//! The latest [documentation can be found here](https://docs.rs/shellcheck_sarif).
//!
//! shellcheck is a popular linter / static analysis tool for shell scripts. More information
//! can be found on the official repository: [https://github.com/koalaman/shellcheck](https://github.com/koalaman/shellcheck)
//!
//! SARIF or the Static Analysis Results Interchange Format is an industry
//! standard format for the output of static analysis tools. More information
//! can be found on the official website: [https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).
//!
//! ## Installation
//!
//! `shellcheck-sarif` may be insalled via `cargo`
//!
//! ```shell
//! cargo install shellcheck-sarif
//! ```
//!
//! ## Usage
//!
//! For most cases, simply run `shellcheck` with `json` output and pipe the
//! results into `shellcheck-sarif`.
//!
//! ## Example
//!
//!```shell
//! shellcheck -f json shellscript.sh | shellcheck-sarif
//! ```
//!
//! If you are using Github Actions, SARIF is useful for integrating with
//! Github Advanced Security (GHAS), which can show code alerts in the
//! "Security" tab of your respository.
//!
//! After uploading `shellcheck-sarif` output to Github, `shellcheck` diagnostics
//! are available in GHAS.
//!
//! ## Example
//!
//! ```yaml
//! on:
//!   workflow_run:
//!     workflows: ["main"]
//!     branches: [main]
//!     types: [completed]
//!
//! name: sarif
//!
//! jobs:
//!   upload-sarif:
//!     runs-on: ubuntu-latest
//!     if: ${{ github.ref == 'refs/heads/main' }}
//!     steps:
//!       - uses: actions/checkout@v2
//!       - uses: actions-rs/toolchain@v1
//!         with:
//!           profile: minimal
//!           toolchain: stable
//!           override: true
//!       - uses: Swatinem/rust-cache@v1
//!       - run: cargo install shellcheck-sarif sarif-fmt
//!       - run:
//!           shellcheck -f json shellscript.sh |
//!           shellcheck-sarif | tee results.sarif | sarif-fmt
//!       - name: Upload SARIF file
//!         uses: github/codeql-action/upload-sarif@v1
//!         with:
//!           sarif_file: results.sarif
//! ```
//!

use anyhow::Result;
use clap::{App, Arg};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

fn main() -> Result<()> {
  let matches = App::new("shellcheck-sarif")
    .about("Convert shellcheck warnings into SARIF")
    .after_help(
      "The expected input is generated by running 'shellcheck -f json'.",
    )
    .version(env!("CARGO_PKG_VERSION"))
    .arg(
      Arg::new("input")
        .help("input file; reads from stdin if none is given")
        .takes_value(true),
    )
    .arg(
      Arg::new("output")
        .help("output file; writes to stdout if none is given")
        .short('o')
        .long("output")
        .takes_value(true),
    )
    .get_matches();

  let read = match matches.value_of_os("input").map(Path::new) {
    Some(path) => Box::new(File::open(path)?) as Box<dyn Read>,
    None => Box::new(std::io::stdin()) as Box<dyn Read>,
  };
  let reader = BufReader::new(read);

  let write = match matches.value_of_os("output").map(Path::new) {
    Some(path) => Box::new(File::create(path)?) as Box<dyn Write>,
    None => Box::new(std::io::stdout()) as Box<dyn Write>,
  };
  let writer = BufWriter::new(write);

  serde_sarif::converters::shellcheck::parse_to_writer(reader, writer)
}
