use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use egui::TextureHandle;

use crate::analyzer::{AnalyzerRegistry, PdfAnalysisResult};
use crate::config::Config;
use crate::error::Result;
use crate::output::{OutputData, OutputRegistry};
use crate::pdf::{PdfFile, PdfRequest, PdfWorker};

#[derive(Debug, Clone, PartialEq)]
pub enum AppTab {
    PdfList,
    Results,
}

#[derive(Debug, Clone)]
pub enum AppState {
    Ready,
    Analyzing,
    Results,
}

#[derive(Debug, Clone)]
pub struct AnalysisProgress {
    pub current_file: String,
    pub current_analyzer: String,
    pub files_done: usize,
    pub files_total: usize,
}

pub enum AnalysisMessage {
    Progress(AnalysisProgress),
    Complete(Vec<PdfAnalysisResult>),
    Error(String),
}

pub struct LoadedPdf {
    pub file: PdfFile,
    pub texture: Option<TextureHandle>,
}

pub struct App {
    pub state: AppState,
    pub current_tab: AppTab,
    pub pdfs: Vec<LoadedPdf>,
    pub config: Config,
    pub analyzer_registry: AnalyzerRegistry,
    pub output_registry: OutputRegistry,
    pub progress: Option<AnalysisProgress>,
    pub analysis_results: Vec<PdfAnalysisResult>,
    pub output_data: Vec<OutputData>,
    pub show_settings: bool,
    pub errors: Vec<String>,

    // Communication channels
    pub analysis_receiver: Option<Receiver<AnalysisMessage>>,

    // PDF worker thread
    worker: PdfWorker,
}

impl Default for App {
    fn default() -> Self {
        let config = Config::load();
        let mut analyzer_registry = AnalyzerRegistry::default();
        let mut output_registry = OutputRegistry::default();

        analyzer_registry.apply_config(&config);
        output_registry.apply_config(&config);

        let worker = PdfWorker::spawn().expect("Failed to initialize PDF worker");

        Self {
            state: AppState::Ready,
            current_tab: AppTab::PdfList,
            pdfs: Vec::new(),
            config,
            analyzer_registry,
            output_registry,
            progress: None,
            analysis_results: Vec::new(),
            output_data: Vec::new(),
            show_settings: false,
            errors: Vec::new(),
            analysis_receiver: None,
            worker,
        }
    }
}

impl App {
    pub fn add_pdf(&mut self, path: PathBuf) -> Result<()> {
        let file = self.worker.load_pdf(path)?;
        self.pdfs.push(LoadedPdf {
            file,
            texture: None,
        });
        Ok(())
    }

    pub fn remove_pdf(&mut self, index: usize) {
        if index < self.pdfs.len() {
            self.pdfs.remove(index);
        }
    }

    pub fn clear(&mut self) {
        self.pdfs.clear();
        self.analysis_results.clear();
        self.output_data.clear();
        self.progress = None;
        self.state = AppState::Ready;
        self.current_tab = AppTab::PdfList;
        self.errors.clear();
    }

    pub fn start_analysis(&mut self) {
        if self.pdfs.is_empty() {
            return;
        }

        let (progress_tx, progress_rx) = mpsc::channel();
        self.analysis_receiver = Some(progress_rx);
        self.state = AppState::Analyzing;
        self.progress = Some(AnalysisProgress {
            current_file: String::new(),
            current_analyzer: String::new(),
            files_done: 0,
            files_total: self.pdfs.len(),
        });

        let paths: Vec<PathBuf> = self.pdfs.iter().map(|p| p.file.path.clone()).collect();
        let worker_tx = self.worker.sender();

        thread::spawn(move || {
            run_analysis(paths, worker_tx, progress_tx);
        });
    }

    pub fn update_analysis(&mut self) {
        let mut completed = false;

        if let Some(ref receiver) = self.analysis_receiver {
            while let Ok(msg) = receiver.try_recv() {
                match msg {
                    AnalysisMessage::Progress(progress) => {
                        self.progress = Some(progress);
                    }
                    AnalysisMessage::Complete(results) => {
                        self.analysis_results = results;
                        self.output_data = self.output_registry.generate_all(&self.analysis_results);
                        self.state = AppState::Results;
                        self.current_tab = AppTab::Results;
                        completed = true;
                    }
                    AnalysisMessage::Error(e) => {
                        self.errors.push(e);
                    }
                }
            }
        }

        if completed {
            self.analysis_receiver = None;
        }
    }

    pub fn save_config(&mut self) {
        if let Err(e) = self.config.save() {
            self.errors.push(format!("Failed to save config: {}", e));
        }
        self.analyzer_registry.apply_config(&self.config);
        self.output_registry.apply_config(&self.config);
    }
}

fn run_analysis(
    paths: Vec<PathBuf>,
    worker_tx: Sender<PdfRequest>,
    progress_tx: Sender<AnalysisMessage>,
) {
    let mut results = Vec::new();
    let total_files = paths.len();

    for (file_idx, path) in paths.iter().enumerate() {
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        // Send progress update
        let _ = progress_tx.send(AnalysisMessage::Progress(AnalysisProgress {
            current_file: filename.clone(),
            current_analyzer: "Analyzing...".to_string(),
            files_done: file_idx,
            files_total: total_files,
        }));

        // Request analysis from the worker thread
        let (response_tx, response_rx) = oneshot::channel();
        if worker_tx
            .send(PdfRequest::AnalyzePdf {
                path: path.clone(),
                response: response_tx,
            })
            .is_err()
        {
            let _ = progress_tx.send(AnalysisMessage::Error(
                "Worker thread not responding".to_string(),
            ));
            continue;
        }

        match response_rx.recv() {
            Ok(Ok(analysis)) => {
                results.push(PdfAnalysisResult {
                    filename: analysis.filename,
                    path: analysis.path,
                    results: analysis.results,
                    errors: analysis.errors,
                });
            }
            Ok(Err(e)) => {
                let _ = progress_tx.send(AnalysisMessage::Error(format!(
                    "Failed to analyze {}: {}",
                    filename, e
                )));
            }
            Err(_) => {
                let _ = progress_tx.send(AnalysisMessage::Error(format!(
                    "Worker died while analyzing {}",
                    filename
                )));
            }
        }
    }

    let _ = progress_tx.send(AnalysisMessage::Complete(results));
}
