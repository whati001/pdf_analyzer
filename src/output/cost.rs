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
