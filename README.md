# dwag

Drag and drop files/folders from your terminal on Windows

![preview](./assets/preview.avif)

> [!NOTE]
> As of `v1.0.0` this project has been rewritten in Rust. It remains a drop-in replacement with slight improvement.
> The original C# implementation is preserved in the [`csharp`](../../tree/csharp) branch and prior [releases](../../release)

## Installation

### Download

Download executable from latest release.

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
