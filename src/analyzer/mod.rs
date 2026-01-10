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
