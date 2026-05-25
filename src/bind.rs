use wasm_bindgen::prelude::*;
use web_sys::{File, js_sys};
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(module = "/pdf.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    pub async fn extract_pdf_book(file: File) -> Result<JsValue, JsValue>;
}