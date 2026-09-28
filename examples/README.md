# formatdiff - a field-based binary comparison tool

[日本語 (Japanese)](README.ja.md)

A tool that compares two files field-by-field, after parsing them as a given format.

It uses [ImHex](https://github.com/WerWolv/ImHex)'s pattern definitions ([ImHex-Patterns](https://github.com/WerWolv/ImHex-Patterns)).

## Building

Requires Rust 2024 Edition, a C++23-capable compiler, and CMake.

```sh
git clone --recurse https://github.com/couyoh/formatdiff
cd formatdiff
cargo build --release
```

## Usage

To compare `a.bmp` and `b.bmp`:

```
formatdiff --patterns-root vendor/ImHex-Patterns/patterns \
    -I vendor/ImHex-Patterns/includes a.bmp b.bmp
```

By default, the `.hexpat` file to use is determined from the file extension.
To specify it explicitly, use `--format`.

The matching `.hexpat` file is looked up starting from the current directory.
To specify that location explicitly, use `--patterns-root`. This is usually pointed at ImHex-Patterns' `patterns/` directory.

To specify a pattern file directly, use `--pattern`.

To specify a pattern library, use `-I`.

## License

GPL-2.0-only
