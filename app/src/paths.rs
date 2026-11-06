/// Optional deployment prefix. Leave unset for a standalone domain or local use.
pub const BASE_PATH: &str = match option_env!("TRAIL_BASE_PATH") {
    Some(path) => path,
    None => "",
};

pub fn url(path: &str) -> String {
    if path.starts_with('/') && !path.starts_with("//") {
        format!("{BASE_PATH}{path}")
    } else {
        path.to_owned()
    }
}
