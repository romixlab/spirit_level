use std::fs;

pub fn load(path: &str) -> Result<String, std::io::Error> {
    let text = fs::read_to_string(path)?;
    Ok(text)
}

pub fn load_or_default(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

pub fn load_matched(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(text) => text,
        Err(_) => String::new(),
    }
}

/// A plain `Option` unrelated to any external call: `unwrap` here is no file, parse or connection
pub fn first_word(text: &str) -> &str {
    text.split_whitespace().next().unwrap()
}

/// Loaded once and checked a few lines above; the second lookup on the same value is not the root call
pub fn cached(cache: &std::collections::HashMap<String, String>, key: &str) -> String {
    if !cache.contains_key(key) {
        return String::new();
    }
    cache.get(key).unwrap().clone()
}
