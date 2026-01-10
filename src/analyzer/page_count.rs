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
