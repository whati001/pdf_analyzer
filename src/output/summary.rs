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
