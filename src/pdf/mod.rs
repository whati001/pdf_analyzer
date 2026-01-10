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

        Ok(bitmap.as_image().to_rgba8())
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
