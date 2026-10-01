# jsoncfmt

Format JSONC with deterministic indentation, recursively sorted keys, attached
comments, and trailing commas.

TODO: GIF demo

## Usage

Format one file to stdout:

```sh
jsoncfmt format settings.jsonc
```

Replace the file atomically:

```sh
jsoncfmt format --in-place true settings.jsonc
```

Keep object keys in their original order:

```sh
jsoncfmt format --sort-keys false settings.jsonc
```

Use four-space indentation:

```sh
jsoncfmt format --indent-width 4 settings.jsonc
```

The `format` options are:

- `-s, --sort-keys <true|false>`: recursively sort object keys by their decoded,
  case-sensitive names. Defaults to `true`. Duplicate keys retain their
  original relative order.
- `-i, --in-place <true|false>`: atomically replace the input file instead of
  writing to stdout. Defaults to `false`.
- `--indent-width <1..=255>`: spaces per indentation level. Defaults to `2`.

`jsoncfmt` preserves array order and expands every non-empty object and array so
each property or element can end with a trailing comma. Empty containers remain
`{}` and `[]`. Output uses spaces, LF newlines, and one final newline.

### Comments

When sorting moves a property, these move with it:

- comments and blank lines between the preceding property and that property,
  including comments separated from the property by blank lines;
- a comment after the property on the same line;
- comments within the property, such as comments around its colon or inside its
  value.

Comments attached to the containing object or file remain there. This includes
comments on the opening brace's line, comments before a closing brace with no
following property, and comments before or after the root value.

### Input and in-place safety

Input must be UTF-8 JSONC: standard JSON plus `//` and `/* */` comments and
optional trailing commas. JSON5 extensions such as unquoted keys, single-quoted
strings, hexadecimal numbers, `Infinity`, and missing commas are rejected.

Mandatory trailing commas make the output JSONC, not strict JSON. Consumers
must support trailing commas.

In-place formatting completes the entire parse and format operation before
atomically replacing the file. It preserves file permissions and leaves the
original unchanged on formatting failure. When the input path is a symbolic
link, `jsoncfmt` replaces its resolved target and preserves the link. Container
nesting deeper than 254 levels is rejected to avoid a formatter-backend limit.

# Install

- [Homebrew](https://brew.sh/): `brew install --cask bbkane/tap/jsoncfmt`
- [Scoop](https://scoop.sh/):

```
scoop bucket add bbkane https://github.com/bbkane/scoop-bucket
scoop install bbkane/jsoncfmt
```

- Download Mac/Linux/Windows executable: [GitHub releases](https://github.com/bbkane/jsoncfmt/releases)
- Build with [goreleaser](https://goreleaser.com/) after cloning: ` goreleaser release --snapshot --clean`

# Dev notes

Requires `zig` for cross compilation:

```
brew install zig
```

## Snapshot Testing

Install `cargo-insta`:

```bash
cargo install cargo-insta
```

Run snapshot tests:

```bash
cargo insta test
```

Agents: inspect every `.snap.new` file, then accept the snapshots without opening the review TUI:

```bash
cargo insta accept
```

Humans: use the TUI to review snapshots:

```bash
cargo insta review
```

Delete snapshots that are no longer referenced by tests:

```bash
cargo insta test --unreferenced delete
```

Run the remaining validation:

```bash
cargo fmt -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test
taplo fmt --check
```
