#![recursion_limit = "512"]

pub mod app;
pub mod cms;
pub mod components;
pub mod data;
pub mod editorial;
pub mod editorial_pages;
pub mod explorer;
pub mod interiors;
pub mod motion;
pub mod pages;
pub mod paths;
pub mod photography;
pub mod studio;
pub mod support;
pub mod support_model;
#[cfg(feature = "ssr")]
pub mod support_store;
#[cfg(feature = "ssr")]
pub mod support_api;
#[cfg(feature = "ssr")]
pub mod studio_api;
pub mod studio_model;
#[cfg(feature = "ssr")]
pub mod studio_store;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
    if let Some(root) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.document_element()) {
        let _ = root.set_attribute("data-hydrated", "true");
    }
}
