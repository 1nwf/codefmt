# codefmt

Markdown code block formatter.

## Installation

### Cargo

```sh
cargo install codefmt --locked
```

### Homebrew

```sh
brew install codefmt
```

## How this formatter works?
This formatter is optimized to be fast. Instead of naively spawning a child process to format each code block, it groups
all of them together and spawns one format child process for each language.

In order to know which parts of the code belong to what block, when code blocks are grouped together, a special code
comment is inserted between blocks to separate them. This allows us to be able to put each code block back into it's
correct position after it is formatted.

For example, suppose we have the following code blocks in a markdown file:

```rust
fn one() {
    println!("one");
}
```

```rust
fn two() {
    println!("two");
}
```

These two codeblocks will be joined into one block and sent to `rustfmt`:

```rust
fn one() {
    println!("one");
}
// __codefmt__
fn two() {
    println!("two");
}
```

## Configuration
There is a default configuration that exists in [languages.toml](link). Feel free to open a PR to add support for more
languages. In addition, you can define your own configuration and pass it to the cli through the `--config` flag.

The configuration looks as follows:

```toml
[aliases]
# aliases define languages aliases for markdown code blocks.
# in this exmaple, it treats `rs` as `rust` code.
rs = "rust"

[languages]
# language configuration requires two fields, the formatter to run, and the language's comment token
# The reason the comment token is needed is due to how this formatter works. This is explained in the previous section.
rust = { formatter = ["rustfmt"], comment_token = "//" }
zig = { formatter = ["zig", "fmt", "--stdin"], comment_token = "//" }
```
