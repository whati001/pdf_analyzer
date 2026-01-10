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
