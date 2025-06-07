# dwag

Drag and drop files/folders from your terminal on Windows

![preview](./assets/preview.avif)

## Prerequisites

[.NET 9.0 Desktop Runtime](https://dotnet.microsoft.com/en-us/download/dotnet/thank-you/runtime-desktop-9.0.5-windows-x64-installer)

## Installation

### Download

Download executable from latest release.

### Build from source

1. Clone the repo
1. `cd dwag; dotnet publish -c Release -r [win-x64|win-arm64]`
1. Copy build output from `bin\Release\net9.0-windows\[win-x64|win-arm64]\publish`

## Usage

```
Usage: dwag [options] [path]...
Options:
    -m  --move  Move files instead of copying
    -h  --help  Show help
```
