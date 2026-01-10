# PDF Analyzer Application Design

## Overview

A cross-platform desktop application for analyzing PDF files. Users can select multiple PDFs, run configurable analyzers, and view/export results including page counts, color analysis, and cost calculations.

## Technology Stack

- **GUI Framework:** egui (via eframe)
- **PDF Processing:** pdfium-render (with bundled PDFium binary)
- **File Dialogs:** rfd (native cross-platform dialogs)
- **Config Persistence:** serde + toml
- **Platform Paths:** dirs crate
- **Error Handling:** thiserror

## Architecture

```
src/
├── main.rs              # Entry point, app initialization
├── app.rs               # Main App struct, state management, UI routing
├── error.rs             # Custom error types
├── analyzer/
│   ├── mod.rs           # Analyzer trait definition
│   ├── page_count.rs    # Page count analyzer
│   └── color_analysis.rs # B&W vs color analyzer
├── output/
│   ├── mod.rs           # Output trait definition
│   ├── summary.rs       # Total pages, B&W/color counts
│   └── cost.rs          # Cost calculation output
├── config/
│   ├── mod.rs           # Config management, persistence
│   └── ui.rs            # Config UI panel
└── pdf/
    ├── mod.rs           # PDF file handling
    └── thumbnail.rs     # Thumbnail generation
```

## Core State Machine

```
Empty List → Adding PDFs → Ready to Analyze → Analyzing (progress) → Results View
     ↑                                                                    │
     └──────────────────── Clear ────────────────────────────────────────┘
```

## Analyzer System

### Trait Definition

```rust
pub trait Analyzer: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn analyze(&self, pdf: &PdfDocument) -> Result<AnalysisResult, AnalyzerError>;
    fn config_params(&self) -> Vec<ConfigParam>;
    fn apply_config(&mut self, config: &Config);
}
```

### AnalysisResult Enum

```rust
pub enum AnalysisResult {
    PageCount { total: usize },
    ColorAnalysis { bw_pages: usize, color_pages: usize },
}
```

### Initial Analyzers

1. **PageCountAnalyzer** - Counts total pages
2. **ColorAnalysisAnalyzer** - Samples pixels per page to detect B&W vs color

### Color Detection Method

Pixel sampling approach: render each page, sample pixels across the page. If any sampled pixel has non-grayscale values (R != G != B), the page is considered color.

## Output System

### Trait Definition

```rust
pub trait OutputModule: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn generate(&self, results: &AnalysisResults) -> OutputData;
    fn config_params(&self) -> Vec<ConfigParam>;
    fn apply_config(&mut self, config: &Config);
}
```

### OutputData Structure

```rust
pub struct OutputData {
    pub title: String,
    pub per_pdf: Vec<PdfOutputRow>,
    pub totals: Vec<(String, String)>,
    pub copyable_text: String,
}

pub struct PdfOutputRow {
    pub filename: String,
    pub values: Vec<(String, String)>,
}
```

### Initial Outputs

1. **SummaryOutput** - Total pages, B&W pages, color pages (per PDF and totals)
2. **CostOutput** - Cost calculation based on configurable rates

## Configuration System

### Config Structure

```rust
pub struct Config {
    pub analyzers: HashMap<String, HashMap<String, ConfigValue>>,
    pub outputs: HashMap<String, HashMap<String, ConfigValue>>,
}

pub enum ConfigValue {
    Bool(bool),
    Float(f64),
    Int(i64),
    String(String),
}
```

### Persistence Paths

- Linux: `~/.config/pdf_analyzer/config.toml`
- Windows: `%APPDATA%\pdf_analyzer\config.toml`

### Initial Config Parameters

| Module | Key | Type | Default | Description |
|--------|-----|------|---------|-------------|
| CostOutput | cost_bw | f64 | 0.05 | Cost per B&W page |
| CostOutput | cost_color | f64 | 0.15 | Cost per color page |
| CostOutput | show_per_pdf | bool | true | Show per-PDF breakdown |
| SummaryOutput | show_per_pdf | bool | true | Show per-PDF breakdown |

## UI Layout

### Main Window (PDF List Tab)

```
┌─────────────────────────────────────────────────────┐
│  PDF Analyzer                        [⚙ Settings]  │
├─────────────────────────────────────────────────────┤
│  [PDF List]  [Results]                              │
├─────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────┐   │
│  │ [+] Add PDFs                                │   │
│  ├─────────────────────────────────────────────┤   │
│  │ 📄 document1.pdf                    [🗑]    │   │
│  │ 📄 report.pdf                       [🗑]    │   │
│  └─────────────────────────────────────────────┘   │
│                                                     │
│  [Analyze]                                          │
│                                                     │
│  ═══════════════════════════ 45%                   │
│  Analyzing: report.pdf - Color detection...         │
└─────────────────────────────────────────────────────┘
```

### Results Tab

```
┌─────────────────────────────────────────────────────┐
│  Summary                                   [Copy]   │
├─────────────────────────────────────────────────────┤
│  File            │ Pages │ B&W │ Color             │
│  document1.pdf   │ 12    │ 10  │ 2                 │
│  ─────────────────────────────────────────         │
│  Total           │ 20    │ 18  │ 2                 │
├─────────────────────────────────────────────────────┤
│  Cost Calculation                          [Copy]   │
├─────────────────────────────────────────────────────┤
│  Grand Total: €1.20                                │
├─────────────────────────────────────────────────────┤
│                    [Clear & Start Over]             │
└─────────────────────────────────────────────────────┘
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Failed to load PDF: {0}")]
    PdfLoad(String),

    #[error("Failed to render page {page}: {reason}")]
    RenderError { page: usize, reason: String },

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Analyzer '{0}' failed: {1}")]
    AnalyzerError(String, String),
}
```

## Threading Model

- UI runs on main thread (egui requirement)
- Analysis runs in background thread
- Communication via channels for progress updates

```rust
pub struct AnalysisProgress {
    pub current_file: String,
    pub current_analyzer: String,
    pub files_done: usize,
    pub files_total: usize,
}
```

## Error Recovery

- Single PDF load failure: skip file, show warning, continue
- Analyzer failure on file: log warning, show "N/A" in results
- Error summary displayed in results if any issues occurred

## Cross-Platform Considerations

- PDFium binary bundled in executable (self-contained)
- Native file dialogs via rfd
- Platform-specific config paths via dirs crate
- No external runtime dependencies
