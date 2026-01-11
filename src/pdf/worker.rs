use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};

use pdfium_render::prelude::*;

use super::PdfFile;
use crate::analyzer::{AnalysisResult, AnalyzerRegistry};
use crate::error::{AppError, Result};

/// Result of analyzing a single PDF
#[derive(Debug, Clone)]
pub struct SinglePdfAnalysis {
    pub filename: String,
    pub path: String,
    pub results: Vec<AnalysisResult>,
    pub errors: Vec<String>,
}

/// Requests that can be sent to the pdfium worker thread
pub enum PdfRequest {
    /// Load PDF metadata and thumbnail
    LoadPdf {
        path: PathBuf,
        response: oneshot::Sender<Result<PdfFile>>,
    },
    /// Analyze a PDF with all registered analyzers
    AnalyzePdf {
        path: PathBuf,
        response: oneshot::Sender<Result<SinglePdfAnalysis>>,
    },
    /// Shutdown the worker thread
    Shutdown,
}

/// Handle to the pdfium worker thread
pub struct PdfWorker {
    request_tx: Sender<PdfRequest>,
    _handle: JoinHandle<()>,
}

impl PdfWorker {
    /// Spawn the pdfium worker thread
    pub fn spawn() -> Result<Self> {
        let (request_tx, request_rx) = mpsc::channel::<PdfRequest>();

        let handle = thread::spawn(move || {
            // Initialize pdfium ONCE in this thread
            let pdfium = match Self::init_pdfium() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to initialize pdfium: {}", e);
                    return;
                }
            };

            let registry = AnalyzerRegistry::default();

            // Process requests until shutdown or channel closes
            while let Ok(request) = request_rx.recv() {
                match request {
                    PdfRequest::LoadPdf { path, response } => {
                        let result = PdfFile::load(path, &pdfium);
                        let _ = response.send(result);
                    }
                    PdfRequest::AnalyzePdf { path, response } => {
                        let result = Self::analyze_pdf(&pdfium, &registry, &path);
                        let _ = response.send(result);
                    }
                    PdfRequest::Shutdown => break,
                }
            }
        });

        Ok(Self {
            request_tx,
            _handle: handle,
        })
    }

    fn init_pdfium() -> Result<Pdfium> {
        let bindings = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())
            .map_err(|e| AppError::PdfLoad {
                path: "pdfium library".to_string(),
                reason: e.to_string(),
            })?;

        Ok(Pdfium::new(bindings))
    }

    fn analyze_pdf(pdfium: &Pdfium, registry: &AnalyzerRegistry, path: &PathBuf) -> Result<SinglePdfAnalysis> {
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let document = pdfium.load_pdf_from_file(path, None).map_err(|e| {
            AppError::PdfLoad {
                path: path.display().to_string(),
                reason: e.to_string(),
            }
        })?;

        let mut results = Vec::new();
        let mut errors = Vec::new();

        for analyzer in registry.analyzers() {
            match analyzer.analyze(&document, path) {
                Ok(result) => results.push(result),
                Err(e) => errors.push(format!("{}: {}", analyzer.name(), e)),
            }
        }

        Ok(SinglePdfAnalysis {
            filename,
            path: path.display().to_string(),
            results,
            errors,
        })
    }

    /// Get a clone of the request sender (for passing to other threads)
    pub fn sender(&self) -> Sender<PdfRequest> {
        self.request_tx.clone()
    }

    /// Load a PDF file (blocking call)
    pub fn load_pdf(&self, path: PathBuf) -> Result<PdfFile> {
        let (response_tx, response_rx) = oneshot::channel();

        self.request_tx
            .send(PdfRequest::LoadPdf {
                path,
                response: response_tx,
            })
            .map_err(|_| AppError::PdfLoad {
                path: "worker".to_string(),
                reason: "Worker thread not responding".to_string(),
            })?;

        response_rx.recv().map_err(|_| AppError::PdfLoad {
            path: "worker".to_string(),
            reason: "Worker thread died".to_string(),
        })?
    }

    /// Analyze a PDF file (blocking call)
    pub fn analyze_pdf_blocking(&self, path: PathBuf) -> Result<SinglePdfAnalysis> {
        let (response_tx, response_rx) = oneshot::channel();

        self.request_tx
            .send(PdfRequest::AnalyzePdf {
                path,
                response: response_tx,
            })
            .map_err(|_| AppError::PdfLoad {
                path: "worker".to_string(),
                reason: "Worker thread not responding".to_string(),
            })?;

        response_rx.recv().map_err(|_| AppError::PdfLoad {
            path: "worker".to_string(),
            reason: "Worker thread died".to_string(),
        })?
    }

    /// Request shutdown of the worker thread
    pub fn shutdown(&self) {
        let _ = self.request_tx.send(PdfRequest::Shutdown);
    }
}

impl Drop for PdfWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}
