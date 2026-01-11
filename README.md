# PDF Analyzer

A cross-platform desktop PDF analysis tool written in Rust. Batch-load PDF files and perform various analyses including page counting and color detection with an intuitive GUI.

## Features

- **Batch PDF Processing**: Load and analyze multiple PDF files simultaneously
- **Page Count Analysis**: Count total pages per PDF
- **Color Detection**: Identify color vs. black & white pages by sampling pixel data
- **Cost Calculation**: Compute printing costs based on configurable rates
- **Thumbnail Preview**: Visual PDF thumbnails in the file list
- **Configurable Settings**: Adjust analyzer parameters and cost rates via settings panel
- **Copy Results**: Export analysis results to clipboard

## Requirements

### Pdfium Library

This application requires the **pdfium** library for PDF rendering. Pdfium is an open-source PDF rendering engine developed by Google and used in Chromium.

**Source**: https://github.com/chromium/pdfium

The application will search for the library in the following order:
1. **Local directory** - Same directory as the executable (`libpdfium.so` on Linux, `pdfium.dll` on Windows)
2. **System library path** - Standard system library locations

#### Linux

Option A - Local installation:
```bash
# Place libpdfium.so in the same directory as the executable
cp /path/to/libpdfium.so ./
```

Option B - System installation:
```bash
# Install to system library path
sudo cp /path/to/libpdfium.so /usr/lib/
sudo ldconfig
```

#### Windows

Place `pdfium.dll` in the same directory as `pdf_analyzer.exe`.

Pre-built binaries can be obtained from:
- https://github.com/ArtifexSoftware/pdfium-binaries (community builds)

## Building

### Prerequisites

- Rust toolchain (1.70+)
- pdfium library (see above)

### Build from source

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

The executable will be located at `target/release/pdf_analyzer`.

## Cross-Compilation from Linux to Windows

This project uses [cross](https://github.com/cross-rs/cross) for cross-compilation. Cross provides Docker-based cross-compilation toolchains, eliminating the need to manually configure MinGW or other toolchains.

### Prerequisites

1. **Install Docker** (or Podman):
```bash
# Arch Linux
sudo pacman -S docker
sudo systemctl enable --now docker

# Debian/Ubuntu
sudo apt install docker.io
sudo systemctl enable --now docker

# Add your user to the docker group (logout required)
sudo usermod -aG docker $USER
```

2. **Install cross**:
```bash
cargo install cross --git https://github.com/cross-rs/cross
```

### Build for Windows

```bash
cross build --release --target x86_64-pc-windows-gnu
```

The Windows executable will be at `target/x86_64-pc-windows-gnu/release/pdf_analyzer.exe`.

### Packaging for Windows

When distributing the Windows build, include:
- `pdf_analyzer.exe`
- `pdfium.dll` (Windows pdfium library)

Ensure you obtain the Windows version of pdfium (`pdfium.dll`) for distribution.

## Usage

1. Launch the application
2. Click "Add PDFs" to select PDF files for analysis
3. Optionally adjust settings via the settings button
4. Click "Analyze" to process the loaded PDFs
5. View results in the "Results" tab
6. Use "Copy" to export results to clipboard

## Configuration

Settings are persisted in a TOML configuration file:
- **Linux**: `~/.config/pdf_analyzer/config.toml`
- **Windows**: `%APPDATA%\pdf_analyzer\config.toml`

### Configurable Parameters

- **Cost per B&W page**: Default $0.05
- **Cost per color page**: Default $0.15
- **Color detection tolerance**: Pixel RGB variance threshold

## License

See LICENSE file for details.
