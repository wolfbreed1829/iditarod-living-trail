//! Small progressive enhancement shell, independent of the WASM download.
use leptos::prelude::*;

pub fn script(options: &LeptosOptions) -> String {
    let suffix=if option_env!("LEPTOS_OUTPUT_NAME").is_some(){""}else{"_bg"};
    let mut assets = serde_json::json!({"js": format!("/{}/{}.js", options.site_pkg_dir, options.output_name), "wasm": format!("/{}/{}{suffix}.wasm", options.site_pkg_dir, options.output_name)});
    #[cfg(feature = "ssr")]
    {
        let path = std::path::Path::new(options.site_root.as_ref()).join(options.site_pkg_dir.as_ref()).join("runtime.json");
        if let Ok(text) = std::fs::read_to_string(path) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                if value["js"].as_str().is_some() && value["wasm"].as_str().is_some() { assets = value; }
            }
        }
    }
    for kind in ["js","wasm"] {
        if let Some(path)=assets[kind].as_str(){assets[kind]=serde_json::Value::String(crate::paths::url(path));}
    }
    format!("({})({});", include_str!("loading.js"), assets)
}
