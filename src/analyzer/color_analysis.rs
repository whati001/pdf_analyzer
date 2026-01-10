use std::path::Path;

use image::GenericImageView;
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
