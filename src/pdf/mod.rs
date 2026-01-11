pub mod service;
// pub mod worker;

use std::path::{Path, PathBuf};

use image::RgbaImage;
use pdfium_render::prelude::*;

use crate::error::{AppError, Result};

// pub use worker::{PdfRequest, PdfWorker};

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

        let document = pdfium
            .load_pdf_from_file(&path, None)
            .map_err(|e| AppError::PdfLoad {
                path: path.display().to_string(),
                reason: e.to_string(),
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
        let page = document
            .pages()
            .get(page_index as u16)
            .map_err(|e| AppError::RenderError {
                page: page_index,
                reason: e.to_string(),
            })?;

        let render_config = PdfRenderConfig::new()
            .set_target_width(150)
            .set_maximum_height(200);

        let bitmap =
            page.render_with_config(&render_config)
                .map_err(|e| AppError::RenderError {
                    page: page_index,
                    reason: e.to_string(),
                })?;

        Ok(bitmap.as_image().to_rgba8())
    }
}
