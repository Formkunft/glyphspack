# glyphspack

`glyphspack` converts between the  `.glyphs` and `.glyphspackage` file format flavors of the [Glyphs font editor](https://glyphsapp.com).

## Usage

Convert between `.glyphs` and `.glyphspackage` files:

```sh
$ glyphspack SomeFont.glyphspackage
Unpacking SomeFont.glyphspackage into SomeFont.glyphs
$ glyphspack OtherFont.glyphs
Packing OtherFont.glyphs into OtherFont.glyphspackage
```

Write the contents to the standard output:

```sh
$ glyphspack -fo /dev/stdout SomeFont.glyphspackage
```

Options:

- Set the output file name with `-o`/`--out`.
- Overwrite any existing files with `-f`/`--force`.
- Suppress log messages with `-q`/`--quiet`.

Run with `--help` for a complete parameter description.

## Installation

The [releases page](https://github.com/Formkunft/glyphspack/releases) provides pre-built executables for **macOS** (universal binary, works with both Apple silicon and Intel), **Linux** (x64 64 bit), and **Windows** (x64 64 bit).

Alternatively, `glyphspack` can be installed from source with [cargo](https://doc.rust-lang.org/cargo/):

```sh
$ cargo install glyphspack
```

## Alternatives

- In Glyphs, save a file to a different format with _File_ → _Save As…_ → _File Format_.
- A similar tool written in Python is [glyphspkg](https://github.com/jenskutilek/glyphspkg).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
