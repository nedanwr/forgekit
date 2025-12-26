//! # Page Specification Parser
//!
//! Parses human-friendly page specifications into structured data. Supports
//! ranges, keywords, exclusions, and combinations.
//!
//! ## Examples
//!
//! - `"1-5"` → pages 1 through 5
//! - `"1-3,5,7-"` → pages 1-3, page 5, and pages 7 to end
//! - `"-10"` → pages 1 through 10
//! - `"odd"` → all odd-numbered pages
//! - `"even"` → all even-numbered pages
//! - `"!2"` → exclude page 2
//! - `"1-10,!2,!5"` → pages 1-10, excluding 2 and 5
//!
//! ## Why a Custom Parser?
//!
//! External tools (like qpdf) have their own page syntax, but it's not always
//! user-friendly. This parser provides a consistent, intuitive syntax that
//! we then convert to whatever the tool expects.

use crate::utils::error::{ForgeKitError, Result};
use std::fmt;

/// A page specification for selecting pages from a document.
///
/// Pages are 1-indexed (first page is 1, not 0). This matches how users
/// think about pages and how most PDF tools work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageSpec {
    /// Single page number (1-indexed, e.g., `Page(5)` means page 5).
    Page(usize),
    /// Range of pages (inclusive on both ends).
    ///
    /// `end: None` means "to the end" (e.g., `"7-"` means pages 7 to last page).
    Range { start: usize, end: Option<usize> },
    /// All odd-numbered pages (1, 3, 5, 7, ...).
    Odd,
    /// All even-numbered pages (2, 4, 6, 8, ...).
    Even,
    /// First page only.
    First,
    /// Last page only.
    Last,
    /// Exclude pages matching the inner spec.
    ///
    /// For example, `Exclude(Box::new(PageSpec::Page(2)))` means "all pages except page 2".
    Exclude(Box<PageSpec>),
}

impl PageSpec {
    /// Parse a page specification string into a vector of `PageSpec`s.
    ///
    /// The string can contain multiple specs separated by commas. Each spec
    /// can be a number, range, keyword, or exclusion.
    ///
    /// # Examples
    ///
    /// ```
    /// use forgekit_core::utils::pages::PageSpec;
    ///
    /// // Single range
    /// let specs = PageSpec::parse("1-5").unwrap();
    /// assert_eq!(specs.len(), 1);
    ///
    /// // Multiple ranges and keywords
    /// let specs = PageSpec::parse("1-3,5,7-,odd").unwrap();
    /// assert_eq!(specs.len(), 4);
    ///
    /// // With exclusions
    /// let specs = PageSpec::parse("1-10,!2,!5").unwrap();
    /// assert_eq!(specs.len(), 3);
    /// ```
    ///
    /// Note: The examples above use `unwrap()` for brevity. In real code,
    /// you should handle the `Result` appropriately.
    ///
    /// # Errors
    ///
    /// Returns `ForgeKitError::InvalidInput` if the string can't be parsed
    /// (e.g., invalid syntax, negative numbers, etc.).
    pub fn parse(spec: &str) -> Result<Vec<PageSpec>> {
        let mut results = Vec::new();
        let parts: Vec<&str> = spec.split(',').map(|s| s.trim()).collect();

        for part in parts {
            if part.is_empty() {
                continue;
            }

            let spec = if let Some(stripped) = part.strip_prefix('!') {
                // Exclusion
                let inner = Self::parse_single(stripped)?;
                PageSpec::Exclude(Box::new(inner))
            } else {
                Self::parse_single(part)?
            };

            results.push(spec);
        }

        Ok(results)
    }

    fn parse_single(s: &str) -> Result<PageSpec> {
        let s = s.trim();

        // Keywords
        match s.to_lowercase().as_str() {
            "odd" => return Ok(PageSpec::Odd),
            "even" => return Ok(PageSpec::Even),
            "first" => return Ok(PageSpec::First),
            "last" => return Ok(PageSpec::Last),
            _ => {}
        }

        // Range or number
        if s.contains('-') {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() != 2 {
                return Err(ForgeKitError::InvalidInput {
                    path: std::path::PathBuf::new(),
                    reason: format!(
                        "Invalid page spec '{}': expected number, range, or keyword (odd/even/first/last)",
                        s
                    ),
                });
            }

            let start_str = parts[0].trim();
            let end_str = parts[1].trim();

            if start_str.is_empty() && end_str.is_empty() {
                return Err(ForgeKitError::InvalidInput {
                    path: std::path::PathBuf::new(),
                    reason: format!("Invalid page spec '{}': range cannot be empty", s),
                });
            }

            let start = if start_str.is_empty() {
                1 // -10 means 1-10
            } else {
                start_str.parse().map_err(|_| ForgeKitError::InvalidInput {
                    path: std::path::PathBuf::new(),
                    reason: format!(
                        "Invalid page spec '{}': '{}' is not a valid number",
                        s, start_str
                    ),
                })?
            };

            let end = if end_str.is_empty() {
                None // 7- means 7 to end
            } else {
                Some(end_str.parse().map_err(|_| ForgeKitError::InvalidInput {
                    path: std::path::PathBuf::new(),
                    reason: format!(
                        "Invalid page spec '{}': '{}' is not a valid number",
                        s, end_str
                    ),
                })?)
            };

            if let Some(end_val) = end {
                if start > end_val {
                    return Err(ForgeKitError::InvalidInput {
                        path: std::path::PathBuf::new(),
                        reason: format!(
                            "Invalid page spec '{}': start ({}) must be <= end ({})",
                            s, start, end_val
                        ),
                    });
                }
            }

            Ok(PageSpec::Range { start, end })
        } else {
            // Single number
            let num = s.parse().map_err(|_| ForgeKitError::InvalidInput {
                path: std::path::PathBuf::new(),
                reason: format!(
                    "Invalid page spec '{}': expected number, range, or keyword (odd/even/first/last)",
                    s
                ),
            })?;

            if num == 0 {
                return Err(ForgeKitError::InvalidInput {
                    path: std::path::PathBuf::new(),
                    reason: "Page numbers must be >= 1".to_string(),
                });
            }

            Ok(PageSpec::Page(num))
        }
    }

    /// Convert page specs to qpdf page specification format
    /// Returns a string like "1-3,5,7-z" for qpdf --pages argument
    pub fn to_qpdf_pages(specs: &[PageSpec], total_pages: usize) -> Result<String> {
        let mut parts = Vec::new();

        for spec in specs {
            match spec {
                PageSpec::Page(n) => {
                    if *n > total_pages {
                        return Err(ForgeKitError::InvalidInput {
                            path: std::path::PathBuf::new(),
                            reason: format!("Page {} exceeds total pages ({})", n, total_pages),
                        });
                    }
                    parts.push(format!("{}", n));
                }
                PageSpec::Range { start, end } => {
                    let end_val = end.unwrap_or(total_pages);
                    if *start > total_pages {
                        return Err(ForgeKitError::InvalidInput {
                            path: std::path::PathBuf::new(),
                            reason: format!("Page {} exceeds total pages ({})", start, total_pages),
                        });
                    }
                    if end_val > total_pages {
                        return Err(ForgeKitError::InvalidInput {
                            path: std::path::PathBuf::new(),
                            reason: format!(
                                "Page {} exceeds total pages ({})",
                                end_val, total_pages
                            ),
                        });
                    }
                    parts.push(format!("{}-{}", start, end_val));
                }
                PageSpec::Odd => {
                    // qpdf doesn't have odd/even, so we need to expand
                    let odd_pages: Vec<String> = (1..=total_pages)
                        .step_by(2)
                        .map(|n| n.to_string())
                        .collect();
                    parts.extend(odd_pages);
                }
                PageSpec::Even => {
                    let even_pages: Vec<String> = (2..=total_pages)
                        .step_by(2)
                        .map(|n| n.to_string())
                        .collect();
                    parts.extend(even_pages);
                }
                PageSpec::First => {
                    parts.push("1".to_string());
                }
                PageSpec::Last => {
                    parts.push(total_pages.to_string());
                }
                PageSpec::Exclude(_) => {
                    // Exclusions need special handling - for now, we'll expand and filter
                    // This is a simplified implementation
                    return Err(ForgeKitError::InvalidInput {
                        path: std::path::PathBuf::new(),
                        reason: "Exclusions (!) are not yet fully supported in qpdf page spec"
                            .to_string(),
                    });
                }
            }
        }

        Ok(parts.join(","))
    }
}

impl fmt::Display for PageSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PageSpec::Page(n) => write!(f, "{}", n),
            PageSpec::Range { start, end } => {
                if let Some(end_val) = end {
                    write!(f, "{}-{}", start, end_val)
                } else {
                    write!(f, "{}-", start)
                }
            }
            PageSpec::Odd => write!(f, "odd"),
            PageSpec::Even => write!(f, "even"),
            PageSpec::First => write!(f, "first"),
            PageSpec::Last => write!(f, "last"),
            PageSpec::Exclude(inner) => write!(f, "!{}", inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_page() {
        let result = PageSpec::parse("5").unwrap();
        assert_eq!(result, vec![PageSpec::Page(5)]);
    }

    #[test]
    fn test_parse_range() {
        let result = PageSpec::parse("1-5").unwrap();
        assert_eq!(
            result,
            vec![PageSpec::Range {
                start: 1,
                end: Some(5)
            }]
        );
    }

    #[test]
    fn test_parse_open_range() {
        let result = PageSpec::parse("7-").unwrap();
        assert_eq!(
            result,
            vec![PageSpec::Range {
                start: 7,
                end: None
            }]
        );
    }

    #[test]
    fn test_parse_start_range() {
        let result = PageSpec::parse("-10").unwrap();
        assert_eq!(
            result,
            vec![PageSpec::Range {
                start: 1,
                end: Some(10)
            }]
        );
    }

    #[test]
    fn test_parse_keywords() {
        assert_eq!(PageSpec::parse("odd").unwrap(), vec![PageSpec::Odd]);
        assert_eq!(PageSpec::parse("even").unwrap(), vec![PageSpec::Even]);
        assert_eq!(PageSpec::parse("first").unwrap(), vec![PageSpec::First]);
        assert_eq!(PageSpec::parse("last").unwrap(), vec![PageSpec::Last]);
    }

    #[test]
    fn test_parse_exclusion() {
        let result = PageSpec::parse("!2").unwrap();
        assert_eq!(result, vec![PageSpec::Exclude(Box::new(PageSpec::Page(2)))]);
    }

    #[test]
    fn test_parse_complex() {
        let result = PageSpec::parse("1-3,5,7-").unwrap();
        assert_eq!(
            result,
            vec![
                PageSpec::Range {
                    start: 1,
                    end: Some(3)
                },
                PageSpec::Page(5),
                PageSpec::Range {
                    start: 7,
                    end: None
                }
            ]
        );
    }

    #[test]
    fn test_parse_invalid() {
        assert!(PageSpec::parse("xyz").is_err());
        assert!(PageSpec::parse("0").is_err());
        assert!(PageSpec::parse("5-3").is_err()); // start > end
    }

    #[test]
    fn test_to_qpdf_pages() {
        let specs = PageSpec::parse("1-3,5").unwrap();
        let result = PageSpec::to_qpdf_pages(&specs, 10).unwrap();
        assert_eq!(result, "1-3,5");
    }

    #[test]
    fn test_to_qpdf_pages_odd() {
        let specs = PageSpec::parse("odd").unwrap();
        let result = PageSpec::to_qpdf_pages(&specs, 5).unwrap();
        assert_eq!(result, "1,3,5");
    }

    #[test]
    fn test_to_qpdf_pages_even() {
        let specs = PageSpec::parse("even").unwrap();
        let result = PageSpec::to_qpdf_pages(&specs, 5).unwrap();
        assert_eq!(result, "2,4");
    }
}
