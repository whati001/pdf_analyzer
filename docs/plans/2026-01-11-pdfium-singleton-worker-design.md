# Pdfium Singleton Worker Design

## Problem

Pdfium can only be initialized once per process. The current code initializes it twice:
1. Main thread via `App::get_pdf_processor()`
2. Analysis thread via `run_analysis() -> PdfProcessor::new()`

## Solution

Create a dedicated pdfium worker thread that:
- Starts when `App` is created
- Owns the single `Pdfium` instance
- Processes requests via channels
- Sends responses back via oneshot channels

## Architecture

```
┌─────────────┐     PdfRequest      ┌─────────────────┐
│  Main/UI    │ ─────────────────►  │  Pdfium Worker  │
│   Thread    │                     │     Thread      │
│             │ ◄─────────────────  │                 │
└─────────────┘   oneshot response  │  owns Pdfium    │
                                    └─────────────────┘
       │
       │ spawn for analysis
       ▼
┌─────────────┐     PdfRequest
│  Analysis   │ ─────────────────►  (same worker)
│   Thread    │
│             │ ◄─────────────────
└─────────────┘   oneshot response
```

## Message Types

```rust
pub enum PdfRequest {
    LoadPdf {
        path: PathBuf,
        response: oneshot::Sender<Result<PdfFile>>,
    },
    AnalyzePdf {
        path: PathBuf,
        response: oneshot::Sender<Result<SinglePdfAnalysis>>,
    },
    Shutdown,
}

pub struct SinglePdfAnalysis {
    pub filename: String,
    pub path: String,
    pub results: Vec<AnalysisResult>,
    pub errors: Vec<String>,
}
```

## File Changes

| File | Change |
|------|--------|
| `src/pdf/worker.rs` | NEW: PdfWorker, PdfRequest, worker loop |
| `src/pdf/mod.rs` | Remove PdfProcessor, keep PdfFile |
| `src/app.rs` | Replace pdf_processor with worker |
| `Cargo.toml` | Add oneshot dependency |

## Dependencies

```toml
oneshot = "0.1"
```
