# dwag

Drag and drop files/folders from your terminal on Windows

![preview](./assets/preview.avif)

> [!NOTE]
> As of `v1.0.0` this project has been rewritten in Rust. It remains a drop-in replacement with slight improvement.
> The original C# implementation is preserved in the [`csharp`](../../tree/csharp) branch and prior [releases](../../releases/tag/v0.4.0).

## Installation

### Download

Download executable from latest release.

### Install With `cargo-binstall`

```
cargo binstall --git https://github.com/8LWXpg/dwag dwag
```

### Build from Source

1. Clone the repo
1. `cd dwag; cargo install --path .`

## Usage

```shell
Usage: dwag [options] [path]...
Options:
    -m  --move  Move files instead of copying
    -h  --help  Show help
```

### Use With `yazi`

In `keymap.toml`

```toml
[[mgr.prepend_keymap]]
on = '<C-o>'
run = 'shell -- dwag %h'
for = 'windows'
desc = 'Drag files/folders'
```
