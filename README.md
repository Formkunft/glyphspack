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
glyphspack -fo /dev/stdout SomeFont.glyphspackage
```

Options:

- Set the output file name with `-o`/`--out`.
- Overwrite any existing files with `-f`/`--force`.
- Suppress log messages with `-q`/`--quiet`.

Run with `--help` for a complete parameter description.

## Installation

**pip:** `glyphspack` can be installed from [PyPI](https://pypi.org/project/glyphspack/):

```sh
pip install glyphspack
```

**Cargo:** `glyphspack` can be installed from [crates.io](https://crates.io/crates/glyphspack):

```sh
cargo install glyphspack
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
