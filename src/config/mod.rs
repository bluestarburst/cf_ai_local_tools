//! Configuration management system

pub mod interpolation;
pub mod manager;
pub mod validation;

// Re-export main types
pub use interpolation::AdvancedInterpolator;
pub use validation::{
    ConfigValidator, CustomValidator, UrlValidator, ValidationError, ValidationResult,
    ValidationRule,
};
