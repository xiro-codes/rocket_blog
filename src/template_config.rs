//! Template configuration with custom filters for minijinja
//! 
//! This module provides custom template configuration for the rocket_dyn_templates
//! system, specifically adding the truncate filter that is missing in minijinja.

use rocket_dyn_templates::{Template, Engines, minijinja};

/// Truncate filter for minijinja templates
/// 
/// This function mimics the behavior of Jinja2's truncate filter.
/// It truncates a string to a specified length and optionally adds an ellipsis.
fn truncate_filter(
    value: &minijinja::Value,
    length: Option<usize>,
) -> Result<String, minijinja::Error> {
    let s = value.as_str().unwrap_or("");
    let length = length.unwrap_or(255);
    let ellipsis = "...";
    
    if s.len() <= length {
        Ok(s.to_string())
    } else {
        // Find the last space within the length limit to avoid breaking words
        let truncated = &s[..length];
        let final_text = if let Some(last_space) = truncated.rfind(' ') {
            format!("{}{}", &truncated[..last_space], ellipsis)
        } else {
            format!("{}{}", truncated, ellipsis)
        };
        Ok(final_text)
    }
}

/// Configure the template engine with custom filters
pub fn configure_template_engine(engines: &mut Engines) {
    engines.minijinja.add_filter("truncate", truncate_filter);
}

/// Create a Template fairing with custom configuration
pub fn create_template_fairing() -> impl rocket::fairing::Fairing {
    Template::custom(|engines| {
        configure_template_engine(engines);
    })
}