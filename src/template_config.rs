//! Template configuration with custom filters for minijinja
//! 
//! This module provides custom template configuration for the rocket_dyn_templates
//! system, specifically adding the truncate filter that is missing in minijinja.
//!
//! ## Problem Solved
//! 
//! The project was using Jinja2-style `truncate` filters in templates like:
//! - `{{ entry.description | truncate(length=20) }}` in worktime_macros.html.j2
//! - `{{ post.text | truncate(length=160) }}` in blog/detail.html.j2
//! 
//! However, minijinja (the template engine used by rocket_dyn_templates) doesn't
//! include a built-in `truncate` filter, causing template rendering errors.
//!
//! ## Solution
//! 
//! This module implements a custom `truncate` filter that:
//! 1. Truncates text to a specified length
//! 2. Breaks at word boundaries when possible
//! 3. Adds "..." when text is truncated
//! 4. Preserves text unchanged when it's shorter than the limit
//!
//! ## Usage
//! 
//! The filter is automatically configured when using `create_template_fairing()`
//! instead of the default `Template::fairing()` in the Rocket application setup.

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