mod analyzer;
mod app;
mod config;
mod error;
mod output;
mod pdf;

use app::App;
use eframe::egui;

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
        Box::new(|cc| {
            // Set up larger fonts
            let mut style = (*cc.egui_ctx.style()).clone();

            // Increase all font sizes by ~40%
            for (_text_style, font_id) in style.text_styles.iter_mut() {
                font_id.size *= 1.4;
            }

            cc.egui_ctx.set_style(style);

            Ok(Box::new(App::default()))
        }),
    )
}

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
                app::AppTab::PdfList => self.show_pdf_list_tab(ui, ctx),
                app::AppTab::Results => self.show_results_tab(ui, ctx),
            }
        });
    }
}

impl App {
    fn show_pdf_list_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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

                for (idx, loaded_pdf) in self.pdfs.iter_mut().enumerate() {
                    // Lazily create texture from thumbnail if needed
                    if loaded_pdf.texture.is_none() {
                        if let Some(ref thumbnail) = loaded_pdf.file.thumbnail {
                            let size = [thumbnail.width() as usize, thumbnail.height() as usize];
                            let pixels = thumbnail.as_flat_samples();
                            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                            let texture = ctx.load_texture(
                                format!("pdf_thumb_{}", idx),
                                color_image,
                                egui::TextureOptions::LINEAR,
                            );
                            loaded_pdf.texture = Some(texture);
                        }
                    }

                    ui.horizontal(|ui| {
                        // Display thumbnail if available
                        if let Some(ref texture) = loaded_pdf.texture {
                            let size = texture.size_vec2();
                            let scaled_height = 60.0;
                            let scale = scaled_height / size.y;
                            let scaled_size = egui::vec2(size.x * scale, scaled_height);
                            ui.image((texture.id(), scaled_size));
                            ui.add_space(8.0);
                        }

                        ui.vertical(|ui| {
                            ui.label(&loaded_pdf.file.filename);
                            ui.weak(format!("{} pages", loaded_pdf.file.page_count));
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑").clicked() {
                                to_remove = Some(idx);
                            }
                        });
                    });
                    ui.add_space(8.0);
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
