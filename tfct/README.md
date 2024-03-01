# tfct

[![CI](https://img.shields.io/github/actions/workflow/status/06chaynes/tfc-toolset/rust.yml?label=CI&style=for-the-badge)](https://github.com/06chaynes/tfc-toolset/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/tfct?style=for-the-badge)](https://crates.io/crates/tfct)
![Crates.io](https://img.shields.io/crates/l/tfct?style=for-the-badge)

A tool to help manage a toolset that helps manage your deployments


## Minimum Supported Rust Version (MSRV)

1.71.1

## Install

You can find the latest binaries for Linux, macOS, and Windows on the [releases page](https://github.com/06chaynes/tfc-toolset/releases).

### Via Homebrew

```sh
brew install 06chaynes/homebrew-tfct/tfct
```

### Via Cargo

```sh
cargo install tfct
```

### Via Shell Script

Where `<version>` is the version you want to install, e.g. `v0.1.0`.

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/06chaynes/tfc-toolset/releases/download/tfct/<version>/tfct-installer.sh | sh
```

### Via Powershell Script

Where `<version>` is the version you want to install, e.g. `v0.1.0`.

```powershell
irm https://github.com/06chaynes/tfc-toolset/releases/download/tfct/<version>/tfct-installer.ps1 | iex
```

## Documentation

- [Book](https://tfc-toolset.rs/tfct/book)

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](https://github.com/06chaynes/http-cache/blob/main/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](https://github.com/06chaynes/http-cache/blob/main/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

