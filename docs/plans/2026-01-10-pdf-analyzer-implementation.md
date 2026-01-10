# PDF Analyzer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a cross-platform desktop PDF analyzer with modular analyzers, outputs, and persistent configuration.

**Architecture:** egui-based GUI with trait-based analyzer and output systems. Background thread for analysis with progress reporting via channels. Persistent TOML configuration.

**Tech Stack:** Rust, eframe/egui, pdfium-render, rfd, serde, toml, dirs, thiserror, image

---

## Task 1: Project Setup and Dependencies

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs`

**Step 1: Update Cargo.toml with dependencies**

```toml
[package]
name = "pdf_analyzer"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe = "0.29"
egui = "0.29"
egui_extras = { version = "0.29", features = ["image"] }
pdfium-render = { version = "0.8", features = ["image"] }
rfd = "0.15"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
dirs = "5.0"
thiserror = "2.0"
image = "0.25"
```

**Step 2: Update main.rs to verify eframe compiles**

```rust
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "PDF Analyzer",
        options,
        Box::new(|_cc| Ok(Box::new(PdfAnalyzerApp::default()))),
    )
}

#[derive(Default)]
struct PdfAnalyzerApp;

impl eframe::App for PdfAnalyzerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PDF Analyzer");
            ui.label("Setup complete!");
        });
    }
}
```

**Step 3: Build and run to verify**

Run: `cargo run`
Expected: Window opens with "PDF Analyzer" heading

**Step 4: Commit**

```bash
git add Cargo.toml src/main.rs
git commit -m "feat: initial project setup with egui"
```

---

## Task 2: Error Module

**Files:**
- Create: `src/error.rs`
- Modify: `src/main.rs`

**Step 1: Create error.rs with error types**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Failed to load PDF '{path}': {reason}")]
    PdfLoad { path: String, reason: String },

    #[error("Failed to render page {page}: {reason}")]
    RenderError { page: usize, reason: String },

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Analyzer '{analyzer}' failed on '{file}': {reason}")]
    AnalyzerError {
        analyzer: String,
        file: String,
        reason: String,
    },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
```

**Step 2: Add module to main.rs**

Add at top of `src/main.rs`:
```rust
mod error;
```

**Step 3: Build to verify**

Run: `cargo build`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/error.rs src/main.rs
git commit -m "feat: add error module with AppError types"
```

---

## Task 3: Config Module - Core Types

**Files:**
- Create: `src/config/mod.rs`
- Modify: `src/main.rs`

**Step 1: Create config directory and mod.rs**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ConfigValue {
    Bool(bool),
    Float(f64),
    Int(i64),
    String(String),
}

impl ConfigValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(v) => Some(*v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigParam {
    pub key: &'static str,
    pub label: &'static str,
    pub default: ConfigValue,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub analyzers: HashMap<String, HashMap<String, ConfigValue>>,
    #[serde(default)]
    pub outputs: HashMap<String, HashMap<String, ConfigValue>>,
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("pdf_analyzer").join("config.toml"))
    }

    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| fs::read_to_string(&path).ok())
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path().ok_or_else(|| {
            AppError::ConfigError("Could not determine config directory".to_string())
        })?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| AppError::ConfigError(e.to_string()))?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn get_analyzer_value(&self, analyzer_id: &str, key: &str) -> Option<&ConfigValue> {
        self.analyzers.get(analyzer_id)?.get(key)
    }

    pub fn set_analyzer_value(&mut self, analyzer_id: &str, key: &str, value: ConfigValue) {
        self.analyzers
            .entry(analyzer_id.to_string())
            .or_default()
            .insert(key.to_string(), value);
    }

    pub fn get_output_value(&self, output_id: &str, key: &str) -> Option<&ConfigValue> {
        self.outputs.get(output_id)?.get(key)
    }

    pub fn set_output_value(&mut self, output_id: &str, key: &str, value: ConfigValue) {
        self.outputs
            .entry(output_id.to_string())
            .or_default()
            .insert(key.to_string(), value);
    }
}
```

**Step 2: Add config module to main.rs**

Add to module declarations at top:
```rust
mod config;
```

**Step 3: Build to verify**

Run: `cargo build`
Expected: Compiles without errors

**Step 4: Commit**

```bash
mkdir -p src/config
git add src/config/mod.rs src/main.rs
git commit -m "feat: add config module with persistence"
```

---

## Task 4: PDF Module - File Handling

**Files:**
- Create: `src/pdf/mod.rs`
- Modify: `src/main.rs`

**Step 1: Create pdf module with PdfFile struct**

```rust
use std::path::PathBuf;
use std::sync::Arc;

use image::RgbaImage;
use pdfium_render::prelude::*;

use crate::error::{AppError, Result};

pub struct PdfFile {
    pub path: PathBuf,
    pub filename: String,
    pub page_count: usize,
    pub thumbnail: Option<RgbaImage>,
}

impl PdfFile {
    pub fn load(path: PathBuf, pdfium: &Pdfium) -> Result<Self> {
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let document = pdfium.load_pdf_from_file(&path, None).map_err(|e| {
            AppError::PdfLoad {
                path: path.display().to_string(),
                reason: e.to_string(),
            }
        })?;

        let page_count = document.pages().len() as usize;

        // Generate thumbnail from first page
        let thumbnail = Self::generate_thumbnail(&document, 0).ok();

        Ok(Self {
            path,
            filename,
            page_count,
            thumbnail,
        })
    }

    fn generate_thumbnail(document: &PdfDocument, page_index: usize) -> Result<RgbaImage> {
        let page = document.pages().get(page_index as u16).map_err(|e| {
            AppError::RenderError {
                page: page_index,
                reason: e.to_string(),
            }
        })?;

        let render_config = PdfRenderConfig::new()
            .set_target_width(150)
            .set_maximum_height(200);

        let bitmap = page.render_with_config(&render_config).map_err(|e| {
            AppError::RenderError {
                page: page_index,
                reason: e.to_string(),
            }
        })?;

        Ok(bitmap.as_image())
    }
}

pub struct PdfProcessor {
    pdfium: Arc<Pdfium>,
}

impl PdfProcessor {
    pub fn new() -> Result<Self> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                .or_else(|_| Pdfium::bind_to_system_library())
                .map_err(|e| AppError::PdfLoad {
                    path: "pdfium library".to_string(),
                    reason: e.to_string(),
                })?,
        );

        Ok(Self {
            pdfium: Arc::new(pdfium),
        })
    }

    pub fn load_pdf(&self, path: PathBuf) -> Result<PdfFile> {
        PdfFile::load(path, &self.pdfium)
    }

    pub fn pdfium(&self) -> &Pdfium {
        &self.pdfium
    }
}
```

**Step 2: Add pdf module to main.rs**

Add to module declarations:
```rust
mod pdf;
```

**Step 3: Build to verify**

Run: `cargo build`
Expected: Compiles without errors

**Step 4: Commit**

```bash
mkdir -p src/pdf
git add src/pdf/mod.rs src/main.rs
git commit -m "feat: add pdf module with file loading and thumbnails"
```

---

## Task 5: Analyzer Module - Trait and Types

**Files:**
- Create: `src/analyzer/mod.rs`
- Modify: `src/main.rs`

**Step 1: Create analyzer module with trait definition**

```rust
use std::path::Path;

use pdfium_render::prelude::*;

use crate::config::{Config, ConfigParam, ConfigValue};
use crate::error::Result;

pub mod page_count;
pub mod color_analysis;

#[derive(Debug, Clone)]
pub enum AnalysisResult {
    PageCount { total: usize },
    ColorAnalysis { bw_pages: usize, color_pages: usize },
}

#[derive(Debug, Clone)]
pub struct PdfAnalysisResult {
    pub filename: String,
    pub path: String,
    pub results: Vec<AnalysisResult>,
    pub errors: Vec<String>,
}

pub trait Analyzer: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn analyze(&self, document: &PdfDocument, path: &Path) -> Result<AnalysisResult>;
    fn config_params(&self) -> Vec<ConfigParam> {
        vec![]
    }
    fn apply_config(&mut self, _config: &Config) {}
}

pub struct AnalyzerRegistry {
    analyzers: Vec<Box<dyn Analyzer>>,
}

impl AnalyzerRegistry {
    pub fn new() -> Self {
        Self { analyzers: vec![] }
    }

    pub fn register(&mut self, analyzer: Box<dyn Analyzer>) {
        self.analyzers.push(analyzer);
    }

    pub fn analyzers(&self) -> &[Box<dyn Analyzer>] {
        &self.analyzers
    }

    pub fn apply_config(&mut self, config: &Config) {
        for analyzer in &mut self.analyzers {
            analyzer.apply_config(config);
        }
    }

    pub fn all_config_params(&self) -> Vec<(&'static str, &'static str, Vec<ConfigParam>)> {
        self.analyzers
            .iter()
            .map(|a| (a.id(), a.name(), a.config_params()))
            .filter(|(_, _, params)| !params.is_empty())
            .collect()
    }
}

impl Default for AnalyzerRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(page_count::PageCountAnalyzer));
        registry.register(Box::new(color_analysis::ColorAnalysisAnalyzer));
        registry
    }
}
```

**Step 2: Add analyzer module to main.rs**

Add to module declarations:
```rust
mod analyzer;
```

**Step 3: Build to verify (will fail - missing submodules)**

Run: `cargo build`
Expected: Error about missing page_count and color_analysis modules (this is expected, we create them next)

**Step 4: Commit partial progress**

```bash
mkdir -p src/analyzer
git add src/analyzer/mod.rs src/main.rs
git commit -m "feat: add analyzer module with trait definition"
```

---

## Task 6: Page Count Analyzer

**Files:**
- Create: `src/analyzer/page_count.rs`

**Step 1: Create page_count.rs**

```rust
use std::path::Path;

use pdfium_render::prelude::*;

use super::{AnalysisResult, Analyzer};
use crate::error::Result;

pub struct PageCountAnalyzer;

impl Analyzer for PageCountAnalyzer {
    fn id(&self) -> &'static str {
        "page_count"
    }

    fn name(&self) -> &'static str {
        "Page Count"
    }

    fn analyze(&self, document: &PdfDocument, _path: &Path) -> Result<AnalysisResult> {
        let total = document.pages().len() as usize;
        Ok(AnalysisResult::PageCount { total })
    }
}
```

**Step 2: Build to verify (will still fail - missing color_analysis)**

Run: `cargo build`
Expected: Error about missing color_analysis module

**Step 3: Commit**

```bash
git add src/analyzer/page_count.rs
git commit -m "feat: add page count analyzer"
```

---

## Task 7: Color Analysis Analyzer

**Files:**
- Create: `src/analyzer/color_analysis.rs`

**Step 1: Create color_analysis.rs**

```rust
use std::path::Path;

use pdfium_render::prelude::*;

use super::{AnalysisResult, Analyzer};
use crate::error::{AppError, Result};

pub struct ColorAnalysisAnalyzer;

impl ColorAnalysisAnalyzer {
    fn is_page_color(page: &PdfPage) -> Result<bool> {
        let render_config = PdfRenderConfig::new()
            .set_target_width(200)
            .set_maximum_height(300);

        let bitmap = page.render_with_config(&render_config).map_err(|e| {
            AppError::RenderError {
                page: 0,
                reason: e.to_string(),
            }
        })?;

        let image = bitmap.as_image();

        // Sample pixels across the image
        let width = image.width();
        let height = image.height();
        let step_x = (width / 20).max(1);
        let step_y = (height / 20).max(1);

        for y in (0..height).step_by(step_y as usize) {
            for x in (0..width).step_by(step_x as usize) {
                let pixel = image.get_pixel(x, y);
                let [r, g, b, _] = pixel.0;

                // Check if pixel is colored (not grayscale)
                // Allow small tolerance for compression artifacts
                let max_diff = r.abs_diff(g).max(r.abs_diff(b)).max(g.abs_diff(b));
                if max_diff > 10 {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

impl Analyzer for ColorAnalysisAnalyzer {
    fn id(&self) -> &'static str {
        "color_analysis"
    }

    fn name(&self) -> &'static str {
        "Color Analysis"
    }

    fn analyze(&self, document: &PdfDocument, _path: &Path) -> Result<AnalysisResult> {
        let mut bw_pages = 0;
        let mut color_pages = 0;

        for page in document.pages().iter() {
            match Self::is_page_color(&page) {
                Ok(true) => color_pages += 1,
                Ok(false) => bw_pages += 1,
                Err(_) => bw_pages += 1, // Default to B&W on error
            }
        }

        Ok(AnalysisResult::ColorAnalysis {
            bw_pages,
            color_pages,
        })
    }
}
```

**Step 2: Build to verify**

Run: `cargo build`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/analyzer/color_analysis.rs
git commit -m "feat: add color analysis analyzer with pixel sampling"
```

---

## Task 8: Output Module - Trait and Types

**Files:**
- Create: `src/output/mod.rs`
- Modify: `src/main.rs`

**Step 1: Create output module with trait definition**

```rust
use crate::analyzer::PdfAnalysisResult;
use crate::config::{Config, ConfigParam, ConfigValue};

pub mod summary;
pub mod cost;

#[derive(Debug, Clone)]
pub struct OutputRow {
    pub filename: String,
    pub values: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct OutputData {
    pub title: String,
    pub columns: Vec<String>,
    pub per_pdf: Vec<OutputRow>,
    pub totals: Vec<(String, String)>,
    pub copyable_text: String,
}

pub trait OutputModule: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn generate(&self, results: &[PdfAnalysisResult]) -> OutputData;
    fn config_params(&self) -> Vec<ConfigParam> {
        vec![]
    }
    fn apply_config(&mut self, _config: &Config) {}
}

pub struct OutputRegistry {
    outputs: Vec<Box<dyn OutputModule>>,
}

impl OutputRegistry {
    pub fn new() -> Self {
        Self { outputs: vec![] }
    }

    pub fn register(&mut self, output: Box<dyn OutputModule>) {
        self.outputs.push(output);
    }

    pub fn outputs(&self) -> &[Box<dyn OutputModule>] {
        &self.outputs
    }

    pub fn apply_config(&mut self, config: &Config) {
        for output in &mut self.outputs {
            output.apply_config(config);
        }
    }

    pub fn all_config_params(&self) -> Vec<(&'static str, &'static str, Vec<ConfigParam>)> {
        self.outputs
            .iter()
            .map(|o| (o.id(), o.name(), o.config_params()))
            .filter(|(_, _, params)| !params.is_empty())
            .collect()
    }

    pub fn generate_all(&self, results: &[PdfAnalysisResult]) -> Vec<OutputData> {
        self.outputs.iter().map(|o| o.generate(results)).collect()
    }
}

impl Default for OutputRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(summary::SummaryOutput::default()));
        registry.register(Box::new(cost::CostOutput::default()));
        registry
    }
}
```

**Step 2: Add output module to main.rs**

Add to module declarations:
```rust
mod output;
```

**Step 3: Build to verify (will fail - missing submodules)**

Run: `cargo build`
Expected: Error about missing summary and cost modules

**Step 4: Commit partial progress**

```bash
mkdir -p src/output
git add src/output/mod.rs src/main.rs
git commit -m "feat: add output module with trait definition"
```

---

## Task 9: Summary Output Module

**Files:**
- Create: `src/output/summary.rs`

**Step 1: Create summary.rs**

```rust
use crate::analyzer::{AnalysisResult, PdfAnalysisResult};
use crate::config::{Config, ConfigParam, ConfigValue};
use super::{OutputData, OutputModule, OutputRow};

pub struct SummaryOutput {
    show_per_pdf: bool,
}

impl Default for SummaryOutput {
    fn default() -> Self {
        Self { show_per_pdf: true }
    }
}

impl OutputModule for SummaryOutput {
    fn id(&self) -> &'static str {
        "summary"
    }

    fn name(&self) -> &'static str {
        "Summary"
    }

    fn config_params(&self) -> Vec<ConfigParam> {
        vec![ConfigParam {
            key: "show_per_pdf",
            label: "Show per-PDF breakdown",
            default: ConfigValue::Bool(true),
            description: "Display page counts for each individual PDF file",
        }]
    }

    fn apply_config(&mut self, config: &Config) {
        if let Some(ConfigValue::Bool(v)) = config.get_output_value(self.id(), "show_per_pdf") {
            self.show_per_pdf = *v;
        }
    }

    fn generate(&self, results: &[PdfAnalysisResult]) -> OutputData {
        let mut total_pages = 0usize;
        let mut total_bw = 0usize;
        let mut total_color = 0usize;

        let mut per_pdf = Vec::new();

        for result in results {
            let mut pages = 0usize;
            let mut bw = 0usize;
            let mut color = 0usize;

            for analysis in &result.results {
                match analysis {
                    AnalysisResult::PageCount { total } => {
                        pages = *total;
                    }
                    AnalysisResult::ColorAnalysis { bw_pages, color_pages } => {
                        bw = *bw_pages;
                        color = *color_pages;
                    }
                }
            }

            total_pages += pages;
            total_bw += bw;
            total_color += color;

            if self.show_per_pdf {
                per_pdf.push(OutputRow {
                    filename: result.filename.clone(),
                    values: vec![
                        ("Pages".to_string(), pages.to_string()),
                        ("B&W".to_string(), bw.to_string()),
                        ("Color".to_string(), color.to_string()),
                    ],
                });
            }
        }

        let totals = vec![
            ("Total Pages".to_string(), total_pages.to_string()),
            ("Total B&W".to_string(), total_bw.to_string()),
            ("Total Color".to_string(), total_color.to_string()),
        ];

        let mut copyable_text = String::new();
        copyable_text.push_str("=== Page Summary ===\n\n");

        if self.show_per_pdf {
            copyable_text.push_str("Per-PDF Breakdown:\n");
            for row in &per_pdf {
                copyable_text.push_str(&format!(
                    "  {}: {} pages ({} B&W, {} color)\n",
                    row.filename,
                    row.values[0].1,
                    row.values[1].1,
                    row.values[2].1
                ));
            }
            copyable_text.push('\n');
        }

        copyable_text.push_str(&format!("Total: {} pages ({} B&W, {} color)\n",
            total_pages, total_bw, total_color));

        OutputData {
            title: "Page Summary".to_string(),
            columns: vec!["File".to_string(), "Pages".to_string(), "B&W".to_string(), "Color".to_string()],
            per_pdf,
            totals,
            copyable_text,
        }
    }
}
```

**Step 2: Build to verify (will still fail - missing cost)**

Run: `cargo build`
Expected: Error about missing cost module

**Step 3: Commit**

```bash
git add src/output/summary.rs
git commit -m "feat: add summary output module"
```

---

## Task 10: Cost Output Module

**Files:**
- Create: `src/output/cost.rs`

**Step 1: Create cost.rs**

```rust
use crate::analyzer::{AnalysisResult, PdfAnalysisResult};
use crate::config::{Config, ConfigParam, ConfigValue};
use super::{OutputData, OutputModule, OutputRow};

pub struct CostOutput {
    cost_bw: f64,
    cost_color: f64,
    show_per_pdf: bool,
}

impl Default for CostOutput {
    fn default() -> Self {
        Self {
            cost_bw: 0.05,
            cost_color: 0.15,
            show_per_pdf: true,
        }
    }
}

impl OutputModule for CostOutput {
    fn id(&self) -> &'static str {
        "cost"
    }

    fn name(&self) -> &'static str {
        "Cost Calculation"
    }

    fn config_params(&self) -> Vec<ConfigParam> {
        vec![
            ConfigParam {
                key: "cost_bw",
                label: "Cost per B&W page",
                default: ConfigValue::Float(0.05),
                description: "Cost in currency units per black & white page",
            },
            ConfigParam {
                key: "cost_color",
                label: "Cost per color page",
                default: ConfigValue::Float(0.15),
                description: "Cost in currency units per color page",
            },
            ConfigParam {
                key: "show_per_pdf",
                label: "Show per-PDF breakdown",
                default: ConfigValue::Bool(true),
                description: "Display costs for each individual PDF file",
            },
        ]
    }

    fn apply_config(&mut self, config: &Config) {
        if let Some(ConfigValue::Float(v)) = config.get_output_value(self.id(), "cost_bw") {
            self.cost_bw = *v;
        }
        if let Some(ConfigValue::Float(v)) = config.get_output_value(self.id(), "cost_color") {
            self.cost_color = *v;
        }
        if let Some(ConfigValue::Bool(v)) = config.get_output_value(self.id(), "show_per_pdf") {
            self.show_per_pdf = *v;
        }
    }

    fn generate(&self, results: &[PdfAnalysisResult]) -> OutputData {
        let mut total_bw_cost = 0.0f64;
        let mut total_color_cost = 0.0f64;

        let mut per_pdf = Vec::new();

        for result in results {
            let mut bw = 0usize;
            let mut color = 0usize;

            for analysis in &result.results {
                if let AnalysisResult::ColorAnalysis { bw_pages, color_pages } = analysis {
                    bw = *bw_pages;
                    color = *color_pages;
                }
            }

            let bw_cost = bw as f64 * self.cost_bw;
            let color_cost = color as f64 * self.cost_color;
            let file_total = bw_cost + color_cost;

            total_bw_cost += bw_cost;
            total_color_cost += color_cost;

            if self.show_per_pdf {
                per_pdf.push(OutputRow {
                    filename: result.filename.clone(),
                    values: vec![
                        ("B&W Cost".to_string(), format!("{:.2}", bw_cost)),
                        ("Color Cost".to_string(), format!("{:.2}", color_cost)),
                        ("Total".to_string(), format!("{:.2}", file_total)),
                    ],
                });
            }
        }

        let grand_total = total_bw_cost + total_color_cost;

        let totals = vec![
            ("Total B&W Cost".to_string(), format!("{:.2}", total_bw_cost)),
            ("Total Color Cost".to_string(), format!("{:.2}", total_color_cost)),
            ("Grand Total".to_string(), format!("{:.2}", grand_total)),
        ];

        let mut copyable_text = String::new();
        copyable_text.push_str("=== Cost Calculation ===\n\n");
        copyable_text.push_str(&format!("Rates: B&W = {:.2}/page, Color = {:.2}/page\n\n",
            self.cost_bw, self.cost_color));

        if self.show_per_pdf {
            copyable_text.push_str("Per-PDF Breakdown:\n");
            for row in &per_pdf {
                copyable_text.push_str(&format!(
                    "  {}: B&W {}, Color {}, Total {}\n",
                    row.filename,
                    row.values[0].1,
                    row.values[1].1,
                    row.values[2].1
                ));
            }
            copyable_text.push('\n');
        }

        copyable_text.push_str(&format!(
            "Totals: B&W {:.2}, Color {:.2}\nGrand Total: {:.2}\n",
            total_bw_cost, total_color_cost, grand_total
        ));

        OutputData {
            title: "Cost Calculation".to_string(),
            columns: vec!["File".to_string(), "B&W Cost".to_string(), "Color Cost".to_string(), "Total".to_string()],
            per_pdf,
            totals,
            copyable_text,
        }
    }
}
```

**Step 2: Build to verify**

Run: `cargo build`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/output/cost.rs
git commit -m "feat: add cost output module with configurable rates"
```

---

## Task 11: App Module - State and Structure

**Files:**
- Create: `src/app.rs`
- Modify: `src/main.rs`

**Step 1: Create app.rs with state management**

```rust
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;

use egui::TextureHandle;
use pdfium_render::prelude::*;

use crate::analyzer::{AnalyzerRegistry, PdfAnalysisResult, AnalysisResult};
use crate::config::Config;
use crate::error::Result;
use crate::output::{OutputData, OutputRegistry};
use crate::pdf::{PdfFile, PdfProcessor};

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

    // PDF processor (lazy initialized)
    pdf_processor: Option<PdfProcessor>,
}

impl Default for App {
    fn default() -> Self {
        let config = Config::load();
        let mut analyzer_registry = AnalyzerRegistry::default();
        let mut output_registry = OutputRegistry::default();

        analyzer_registry.apply_config(&config);
        output_registry.apply_config(&config);

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
            pdf_processor: None,
        }
    }
}

impl App {
    pub fn get_pdf_processor(&mut self) -> Result<&PdfProcessor> {
        if self.pdf_processor.is_none() {
            self.pdf_processor = Some(PdfProcessor::new()?);
        }
        Ok(self.pdf_processor.as_ref().unwrap())
    }

    pub fn add_pdf(&mut self, path: PathBuf) -> Result<()> {
        let processor = self.get_pdf_processor()?;
        let file = processor.load_pdf(path)?;
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

        let (tx, rx) = mpsc::channel();
        self.analysis_receiver = Some(rx);
        self.state = AppState::Analyzing;
        self.progress = Some(AnalysisProgress {
            current_file: String::new(),
            current_analyzer: String::new(),
            files_done: 0,
            files_total: self.pdfs.len(),
        });

        let paths: Vec<PathBuf> = self.pdfs.iter().map(|p| p.file.path.clone()).collect();
        let analyzer_count = self.analyzer_registry.analyzers().len();

        thread::spawn(move || {
            run_analysis(paths, analyzer_count, tx);
        });
    }

    pub fn update_analysis(&mut self) {
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
                        self.analysis_receiver = None;
                    }
                    AnalysisMessage::Error(e) => {
                        self.errors.push(e);
                    }
                }
            }
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

fn run_analysis(paths: Vec<PathBuf>, analyzer_count: usize, tx: Sender<AnalysisMessage>) {
    let processor = match PdfProcessor::new() {
        Ok(p) => p,
        Err(e) => {
            let _ = tx.send(AnalysisMessage::Error(e.to_string()));
            return;
        }
    };

    let registry = AnalyzerRegistry::default();
    let mut results = Vec::new();
    let total_files = paths.len();

    for (file_idx, path) in paths.iter().enumerate() {
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let document = match processor.pdfium().load_pdf_from_file(path, None) {
            Ok(doc) => doc,
            Err(e) => {
                let _ = tx.send(AnalysisMessage::Error(format!(
                    "Failed to load {}: {}",
                    filename, e
                )));
                continue;
            }
        };

        let mut pdf_results = Vec::new();
        let mut pdf_errors = Vec::new();

        for analyzer in registry.analyzers() {
            let _ = tx.send(AnalysisMessage::Progress(AnalysisProgress {
                current_file: filename.clone(),
                current_analyzer: analyzer.name().to_string(),
                files_done: file_idx,
                files_total: total_files,
            }));

            match analyzer.analyze(&document, path) {
                Ok(result) => pdf_results.push(result),
                Err(e) => pdf_errors.push(format!("{}: {}", analyzer.name(), e)),
            }
        }

        results.push(PdfAnalysisResult {
            filename,
            path: path.display().to_string(),
            results: pdf_results,
            errors: pdf_errors,
        });
    }

    let _ = tx.send(AnalysisMessage::Complete(results));
}
```

**Step 2: Update main.rs to use app module**

Replace entire content of `src/main.rs`:
```rust
mod analyzer;
mod app;
mod config;
mod error;
mod output;
mod pdf;

use eframe::egui;
use app::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "PDF Analyzer",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for analysis updates
        self.update_analysis();

        // Request repaint during analysis
        if matches!(self.state, app::AppState::Analyzing) {
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("PDF Analyzer");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙ Settings").clicked() {
                        self.show_settings = true;
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("App structure ready - UI coming next task");
        });
    }
}
```

**Step 3: Build to verify**

Run: `cargo build`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: add app module with state management and analysis threading"
```

---

## Task 12: Main UI - Tab Navigation and PDF List

**Files:**
- Modify: `src/main.rs`

**Step 1: Update main.rs with full UI implementation**

Replace the `impl eframe::App for App` block in `src/main.rs`:

```rust
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_analysis();

        if matches!(self.state, app::AppState::Analyzing) {
            ctx.request_repaint();
        }

        // Settings window
        if self.show_settings {
            self.show_settings_window(ctx);
        }

        // Top panel with title and settings button
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("PDF Analyzer");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙ Settings").clicked() {
                        self.show_settings = true;
                    }
                });
            });

            ui.add_space(4.0);

            // Tab bar
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, app::AppTab::PdfList, "PDF List");
                ui.selectable_value(&mut self.current_tab, app::AppTab::Results, "Results");
            });
        });

        // Bottom panel for progress and actions
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(8.0);

            // Show errors if any
            if !self.errors.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚠").color(egui::Color32::YELLOW));
                    ui.label(&self.errors[self.errors.len() - 1]);
                    if ui.button("Clear").clicked() {
                        self.errors.clear();
                    }
                });
                ui.add_space(4.0);
            }

            // Progress bar during analysis
            if let Some(ref progress) = self.progress {
                if matches!(self.state, app::AppState::Analyzing) {
                    let fraction = progress.files_done as f32 / progress.files_total as f32;
                    ui.add(egui::ProgressBar::new(fraction).show_percentage());
                    ui.label(format!(
                        "Analyzing: {} - {}",
                        progress.current_file, progress.current_analyzer
                    ));
                }
            }

            ui.add_space(4.0);
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                app::AppTab::PdfList => self.show_pdf_list_tab(ui),
                app::AppTab::Results => self.show_results_tab(ui, ctx),
            }
        });
    }
}

impl App {
    fn show_pdf_list_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("+ Add PDFs").clicked() {
                if let Some(paths) = rfd::FileDialog::new()
                    .add_filter("PDF files", &["pdf"])
                    .set_title("Select PDF files")
                    .pick_files()
                {
                    for path in paths {
                        if let Err(e) = self.add_pdf(path) {
                            self.errors.push(e.to_string());
                        }
                    }
                }
            }

            ui.add_space(16.0);

            let can_analyze = !self.pdfs.is_empty() && matches!(self.state, app::AppState::Ready | app::AppState::Results);
            ui.add_enabled_ui(can_analyze, |ui| {
                if ui.button("▶ Analyze").clicked() {
                    self.start_analysis();
                }
            });

            if !self.pdfs.is_empty() {
                ui.add_space(16.0);
                if ui.button("Clear All").clicked() {
                    self.clear();
                }
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        if self.pdfs.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(egui::RichText::new("No PDF files added").size(16.0).weak());
                ui.label("Click '+ Add PDFs' to select files");
            });
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut to_remove = None;

                for (idx, loaded_pdf) in self.pdfs.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("📄 {}", loaded_pdf.file.filename));
                        ui.weak(format!("({} pages)", loaded_pdf.file.page_count));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑").clicked() {
                                to_remove = Some(idx);
                            }
                        });
                    });
                    ui.add_space(4.0);
                }

                if let Some(idx) = to_remove {
                    self.remove_pdf(idx);
                }
            });
        }
    }

    fn show_results_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.output_data.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(egui::RichText::new("No results yet").size(16.0).weak());
                ui.label("Add PDFs and click 'Analyze' to see results");
            });
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for output in &self.output_data {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&output.title);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("📋 Copy").clicked() {
                                ctx.copy_text(output.copyable_text.clone());
                            }
                        });
                    });

                    ui.add_space(8.0);

                    if !output.per_pdf.is_empty() {
                        egui::Grid::new(format!("grid_{}", output.title))
                            .striped(true)
                            .min_col_width(80.0)
                            .show(ui, |ui| {
                                // Header row
                                for col in &output.columns {
                                    ui.strong(col);
                                }
                                ui.end_row();

                                // Data rows
                                for row in &output.per_pdf {
                                    ui.label(&row.filename);
                                    for (_, value) in &row.values {
                                        ui.label(value);
                                    }
                                    ui.end_row();
                                }
                            });

                        ui.add_space(8.0);
                        ui.separator();
                    }

                    // Totals
                    ui.add_space(4.0);
                    for (label, value) in &output.totals {
                        ui.horizontal(|ui| {
                            ui.strong(format!("{}:", label));
                            ui.label(value);
                        });
                    }
                });

                ui.add_space(16.0);
            }

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("🔄 Clear & Start Over").clicked() {
                    self.clear();
                }
            });
        });
    }

    fn show_settings_window(&mut self, ctx: &egui::Context) {
        let mut show_settings = self.show_settings;

        egui::Window::new("Settings")
            .open(&mut show_settings)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut config_changed = false;

                    // Analyzer settings
                    ui.collapsing("Analyzers", |ui| {
                        let analyzer_params = self.analyzer_registry.all_config_params();
                        if analyzer_params.is_empty() {
                            ui.label("No configurable analyzers");
                        } else {
                            for (id, name, params) in analyzer_params {
                                ui.group(|ui| {
                                    ui.strong(name);
                                    for param in params {
                                        config_changed |= self.render_config_param(ui, id, &param, true);
                                    }
                                });
                            }
                        }
                    });

                    ui.add_space(8.0);

                    // Output settings
                    ui.collapsing("Outputs", |ui| {
                        let output_params = self.output_registry.all_config_params();
                        for (id, name, params) in output_params {
                            ui.group(|ui| {
                                ui.strong(name);
                                for param in params {
                                    config_changed |= self.render_config_param(ui, id, &param, false);
                                }
                            });
                        }
                    });

                    if config_changed {
                        self.save_config();
                    }
                });
            });

        self.show_settings = show_settings;
    }

    fn render_config_param(
        &mut self,
        ui: &mut egui::Ui,
        module_id: &str,
        param: &config::ConfigParam,
        is_analyzer: bool,
    ) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label(param.label);
            ui.add_space(8.0);

            match &param.default {
                config::ConfigValue::Bool(default) => {
                    let current = if is_analyzer {
                        self.config.get_analyzer_value(module_id, param.key)
                    } else {
                        self.config.get_output_value(module_id, param.key)
                    }
                    .and_then(|v| v.as_bool())
                    .unwrap_or(*default);

                    let mut value = current;
                    if ui.checkbox(&mut value, "").changed() {
                        if is_analyzer {
                            self.config.set_analyzer_value(
                                module_id,
                                param.key,
                                config::ConfigValue::Bool(value),
                            );
                        } else {
                            self.config.set_output_value(
                                module_id,
                                param.key,
                                config::ConfigValue::Bool(value),
                            );
                        }
                        changed = true;
                    }
                }
                config::ConfigValue::Float(default) => {
                    let current = if is_analyzer {
                        self.config.get_analyzer_value(module_id, param.key)
                    } else {
                        self.config.get_output_value(module_id, param.key)
                    }
                    .and_then(|v| v.as_float())
                    .unwrap_or(*default);

                    let mut value = current;
                    if ui
                        .add(egui::DragValue::new(&mut value).speed(0.01).range(0.0..=1000.0))
                        .changed()
                    {
                        if is_analyzer {
                            self.config.set_analyzer_value(
                                module_id,
                                param.key,
                                config::ConfigValue::Float(value),
                            );
                        } else {
                            self.config.set_output_value(
                                module_id,
                                param.key,
                                config::ConfigValue::Float(value),
                            );
                        }
                        changed = true;
                    }
                }
                _ => {
                    ui.label("(unsupported type)");
                }
            }
        });

        if !param.description.is_empty() {
            ui.weak(param.description);
        }

        changed
    }
}
```

**Step 2: Add rfd import to main.rs**

At the top of main.rs, update imports:
```rust
mod analyzer;
mod app;
mod config;
mod error;
mod output;
mod pdf;

use app::App;
use eframe::egui;
```

**Step 3: Build and run**

Run: `cargo run`
Expected: Full UI appears with tabs, file picker, settings

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: implement full UI with tabs, PDF list, results, and settings"
```

---

## Task 13: Final Polish and Testing

**Files:**
- Modify: `Cargo.toml` (add metadata)
- Modify: `src/main.rs` (fix any issues)

**Step 1: Add package metadata to Cargo.toml**

Update the `[package]` section:
```toml
[package]
name = "pdf_analyzer"
version = "0.1.0"
edition = "2021"
description = "Cross-platform PDF analyzer with page counting and cost calculation"
license = "MIT"
authors = ["Your Name"]

[dependencies]
eframe = "0.29"
egui = "0.29"
egui_extras = { version = "0.29", features = ["image"] }
pdfium-render = { version = "0.8", features = ["image"] }
rfd = "0.15"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
dirs = "5.0"
thiserror = "2.0"
image = "0.25"

[profile.release]
opt-level = 3
lto = true
```

**Step 2: Build release version**

Run: `cargo build --release`
Expected: Compiles without errors, creates optimized binary

**Step 3: Test the application**

Run: `cargo run --release`
Expected:
1. Window opens
2. Can add PDF files via file picker
3. Can remove PDFs from list
4. Analyze button processes PDFs
5. Results show in Results tab
6. Copy button works
7. Settings window shows config options
8. Clear button resets app

**Step 4: Final commit**

```bash
git add Cargo.toml
git commit -m "feat: add package metadata and release profile"
```

---

## Summary

The implementation creates a modular PDF analyzer with:

- **Error handling**: Custom error types in `src/error.rs`
- **Config system**: Persistent TOML config in `src/config/mod.rs`
- **PDF processing**: File loading and thumbnails in `src/pdf/mod.rs`
- **Analyzer trait**: Extensible analysis system in `src/analyzer/`
  - PageCountAnalyzer
  - ColorAnalysisAnalyzer
- **Output trait**: Extensible output system in `src/output/`
  - SummaryOutput
  - CostOutput (with configurable rates)
- **App state**: State machine and threading in `src/app.rs`
- **UI**: Tab-based egui interface in `src/main.rs`

To add new analyzers: implement `Analyzer` trait, register in `AnalyzerRegistry::default()`
To add new outputs: implement `OutputModule` trait, register in `OutputRegistry::default()`
