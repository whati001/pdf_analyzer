use crossbeam_channel as chan;
use once_cell::sync::Lazy;
use pdfium_render::prelude::*;
use std::{
    cell::OnceCell,
    path::{Path, PathBuf},
    sync::OnceLock,
    thread,
};

use crate::{
    analyzer::{AnalysisResult, AnalyzerRegistry},
    app::App,
    error::AppError,
    pdf::PdfFile,
};

/// A job to be executed on the Pdfium worker thread.
/// It receives a mutable reference to Pdfium.
type Job = Box<dyn FnOnce(&mut Pdfium) + Send + 'static>;

/// Requests that can be sent to the pdfium service thread
pub enum PdfSerivceRequest {
    /// A job to be executed on the Pdfium worker thread
    Job(Job),

    /// Shutdown the worker thread
    Shutdown,
}

/// A handle you can clone and use from any thread.
#[derive(Clone, Debug)]
pub struct PdfiumService {
    tx: chan::Sender<PdfSerivceRequest>,
    // worker: &'static OnceLock<PdfiumWorker>,
}

#[derive(Debug)]
pub struct PdfiumWorker {
    handle: thread::JoinHandle<()>,
    service: PdfiumService,
}

// /// A handle you can clone and use from any thread.
// #[derive(Clone)]
// pub struct PdfiumService {
//     tx: chan::Sender<Job>,
// }

// /// The message sent to the worker thread.
// /// It contains a boxed function that will be executed on the worker thread,
// /// receiving `&mut Pdfium`, and returning a boxed "any" result.
// ///
// /// Why `Box<dyn Any + Send>`?
// /// So different calls can return different types, while remaining type-safe
// /// at the call site via downcasting.

/// Global singleton service handle (optional).
static PDFIUM_WORKER: OnceLock<PdfiumWorker> = OnceLock::new();

impl PdfiumWorker {
    pub fn spawn() -> crate::error::Result<()> {
        let (tx, rx) = chan::unbounded::<PdfSerivceRequest>();

        // Spawn the dedicated worker thread.
        thread::Builder::new()
            .name("pdfium-worker".to_string())
            .spawn(move || {
                // Create Pdfium INSIDE the worker thread.
                let pdfium_binding =
                    Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                        .or_else(|_| Pdfium::bind_to_system_library())
                        .map_err(|e| AppError::PdfLibrary {
                            reason: e.to_string(),
                        })
                        .expect("Failed to load Pdfium library");

                let mut pdfium = Pdfium::new(pdfium_binding);

                // Process jobs forever.
                for job in rx.iter() {
                    match job {
                        PdfSerivceRequest::Job(j) => j(&mut pdfium),
                        PdfSerivceRequest::Shutdown => break,
                    }
                }
            })
            .map(|handle| {
                // Store the service handle globally.
                let worker = PdfiumWorker {
                    handle,
                    service: PdfiumService {
                        tx,
                        // worker: &PDFIUM_WORKER,
                    },
                };

                PDFIUM_WORKER
                    .set(worker)
                    .expect("PdfiumWorker already initialized");

                Ok(())
            })?
    }

    /// Get the global PdfiumService handle.
    pub fn service() -> crate::error::Result<PdfiumService> {
        PDFIUM_WORKER
            .get()
            .map(|worker| worker.service.clone())
            .ok_or_else(|| AppError::PdfLibrary {
                reason: "Failed to get PdfiumService, verify if PdfiumWorker is initialized"
                    .to_string(),
            })
    }
}

/// Result of analyzing a single PDF
#[derive(Debug, Clone)]
pub struct SinglePdfAnalysis {
    pub filename: String,
    pub path: String,
    pub results: Vec<AnalysisResult>,
    pub errors: Vec<String>,
}

impl PdfiumService {
    pub fn sender(&self) -> chan::Sender<PdfSerivceRequest> {
        self.tx.clone()
    }

    pub fn load_pdf(&self, path: PathBuf) -> crate::error::Result<PdfFile> {
        self.call(|pdfium| PdfFile::load(path, &pdfium))
    }

    pub fn analyze_pdf(&self, path: PathBuf) -> crate::error::Result<SinglePdfAnalysis> {
        self.call(|pdfium| {
            let registry = AnalyzerRegistry::default();
            Self::analyze_pdf_by_registry(pdfium, &registry, path)
        })
    }

    fn analyze_pdf_by_registry(
        pdfium: &Pdfium,
        registry: &AnalyzerRegistry,
        path: PathBuf,
    ) -> crate::error::Result<SinglePdfAnalysis> {
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let document = pdfium
            .load_pdf_from_file(&path, None)
            .map_err(|e| AppError::PdfLoad {
                path: path.display().to_string(),
                reason: e.to_string(),
            })?;

        let mut results = Vec::new();
        let mut errors = Vec::new();

        for analyzer in registry.analyzers() {
            match analyzer.analyze(&document, &path) {
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

    /// Run a function on the Pdfium worker thread and get a typed result back.
    ///
    /// This is the ergonomic API you’ll use everywhere.
    pub fn call<R, F>(&self, f: F) -> R
    where
        R: Send + 'static,
        F: FnOnce(&mut Pdfium) -> R + Send + 'static,
    {
        // One-shot channel for the response.
        let (rtx, rrx) = chan::bounded::<R>(1);

        // Wrap the user function into a Job and send it to the worker.
        let job: Job = Box::new(move |pdfium: &mut Pdfium| {
            let result = f(pdfium);
            // Ignore send errors if caller dropped receiver.
            let _ = rtx.send(result);
        });

        self.tx
            .send(PdfSerivceRequest::Job(job))
            .expect("Pdfium worker thread seems to have stopped");

        // Wait for the response.
        rrx.recv().expect("Pdfium worker did not return a result")
    }

    /// Fire-and-forget variant (no result).
    pub fn cast<F>(&self, f: F)
    where
        F: FnOnce(&mut Pdfium) + Send + 'static,
    {
        let job: Job = Box::new(move |pdfium: &mut Pdfium| f(pdfium));
        self.tx
            .send(PdfSerivceRequest::Job(job))
            .expect("Pdfium worker thread seems to have stopped");
    }

    /// Shutdown the worker thread
    pub fn shutdown(&self) {
        let _ = self.tx.send(PdfSerivceRequest::Shutdown);
    }
}
