mod config;
mod error;
mod pdf;

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
