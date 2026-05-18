//! Variable detection and expansion for DNF configuration values.
//!
//! DNF supports shell-like variable substitution in `.repo` file values.
//! This module provides two functions:
//!
//! - [`expand_variables`] — substitutes `$var`, `${var}`, `${var:-default}`,
//!   and `${var:+alt}` patterns with values from a user-provided map.
//! - [`detect_variables`] — scans a string and returns the names of all
//!   variables referenced (without expanding them).
//!
//! # Supported Syntax
//!
//! | Pattern               | Behavior                                      |
//! |-----------------------|-----------------------------------------------|
//! | `$releasever`         | Simple variable reference                      |
//! | `${basearch}`         | Braced variable reference                      |
//! | `${var:-default}`     | Use `default` if `var` is unset or empty       |
//! | `${var:+alt}`         | Use `alt` if `var` is set and non-empty        |
//! | `\$releasever`        | Escaped dollar sign (literal `$releasever`)    |
//!
//! Recursive expansion is supported up to a maximum depth of 32.

use crate::error::ExpandError;
use std::collections::{HashMap, HashSet};

const MAX_EXPRESSION_DEPTH: u32 = 32;

/// Expand DNF variables in a string using the given substitution map.
///
/// Supports `$var`, `${var}`, `${var:-default}`, and `${var:+alt}` syntax.
/// Backslash-escaped `\$` sequences are treated as literal dollar signs.
/// Recursive expansion is supported up to a depth of 32.
///
/// # Errors
///
/// Returns [`ExpandError::VariableNotFound`] if a referenced variable is not
/// present in the substitution map (and no `:-default` fallback is provided).
/// Returns [`ExpandError::MaxDepthExceeded`] if the expansion recursion limit
/// is hit.
/// Returns [`ExpandError::MalformedExpression`] for syntactically invalid
/// variable expressions (e.g., a bare `$` at end of string).
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use dnf_repofile::expand_variables;
///
/// let mut vars = HashMap::new();
/// vars.insert("releasever".to_string(), "38".to_string());
/// vars.insert("basearch".to_string(), "x86_64".to_string());
///
/// let expanded = expand_variables(
///     "fedora-$releasever-$basearch",
///     &vars,
/// ).unwrap();
/// assert_eq!(expanded, "fedora-38-x86_64");
/// ```
///
/// ```
/// use std::collections::HashMap;
/// use dnf_repofile::expand_variables;
///
/// let vars = HashMap::new();
///
/// // Default value substitution
/// let expanded = expand_variables(
///     "${releasever:-39}",
///     &vars,
/// ).unwrap();
/// assert_eq!(expanded, "39");
/// ```
pub fn expand_variables(
    input: &str,
    vars: &HashMap<String, String>,
) -> std::result::Result<String, ExpandError> {
    let mut used = HashSet::new();
    expand_recursive(input, vars, 0, &mut used)
}

fn expand_recursive(
    input: &str,
    vars: &HashMap<String, String>,
    depth: u32,
    used: &mut HashSet<String>,
) -> std::result::Result<String, ExpandError> {
    if depth > MAX_EXPRESSION_DEPTH {
        return Err(ExpandError::MaxDepthExceeded {
            depth,
            expr: input.to_owned(),
        });
    }

    let chars: Vec<char> = input.chars().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            result.push(chars[i + 1]);
            i += 2;
            continue;
        }

        if chars[i] == '$' && i + 1 < chars.len() {
            let start = i;
            let mut j = i + 1;
            let mut name = String::new();
            let mut is_braced = false;

            if j < chars.len() && chars[j] == '{' {
                is_braced = true;
                j += 1;
            }

            while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                name.push(chars[j]);
                j += 1;
            }

            if name.is_empty() {
                return Err(ExpandError::MalformedExpression {
                    expr: chars[start..].iter().collect(),
                });
            }

            let default_val: Option<String> = if is_braced && j < chars.len() {
                if j + 1 < chars.len()
                    && chars[j] == ':'
                    && (chars[j + 1] == '-' || chars[j + 1] == '+')
                {
                    let is_default = chars[j + 1] == '-';
                    j += 2;
                    let mut val = String::new();
                    let mut depth_count = 1u32;
                    while j < chars.len() && depth_count > 0 {
                        if chars[j] == '{' {
                            depth_count += 1;
                        } else if chars[j] == '}' {
                            depth_count -= 1;
                            if depth_count == 0 {
                                j += 1;
                                break;
                            }
                        }
                        val.push(chars[j]);
                        j += 1;
                    }
                    let var_val = vars.get(&name);
                    Some(if is_default {
                        if var_val.is_none_or(|v| v.is_empty()) {
                            val
                        } else {
                            var_val.cloned().unwrap_or(val)
                        }
                    } else if var_val.is_some_and(|v| !v.is_empty()) {
                        val
                    } else {
                        String::new()
                    })
                } else {
                    None
                }
            } else {
                None
            };

            i = j;

            // Consume closing brace for braced variables
            if is_braced && i < chars.len() && chars[i] == '}' {
                i += 1;
            }

            if let Some(dv) = default_val {
                let expanded = expand_recursive(&dv, vars, depth + 1, used)?;
                result.push_str(&expanded);
            } else {
                let replacement = vars
                    .get(&name)
                    .ok_or_else(|| ExpandError::VariableNotFound { name: name.clone() })?;
                result.push_str(replacement);
            }

            used.insert(name);
            continue;
        }

        result.push(chars[i]);
        i += 1;
    }

    Ok(result)
}

/// Detect all DNF variable references in a string without expanding them.
///
/// Scans the input for `$var` and `${var}` patterns and returns the variable
/// names. Each variable name is returned once per occurrence (duplicates
/// are not deduplicated).
///
/// Backslash-escaped `\$` sequences are ignored.
///
/// # Examples
///
/// ```
/// use dnf_repofile::detect_variables;
///
/// let vars = detect_variables("$releasever/${basearch}/repo");
/// assert_eq!(vars, vec!["releasever", "basearch"]);
/// ```
///
/// ```
/// use dnf_repofile::detect_variables;
///
/// // Escaped dollar signs are skipped
/// let vars = detect_variables("\\$releasever");
/// assert!(vars.is_empty());
/// ```
pub fn detect_variables(input: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            i += 2;
            continue;
        }

        if chars[i] == '$' && i + 1 < chars.len() {
            let mut j = i + 1;
            let mut name = String::new();

            if j < chars.len() && chars[j] == '{' {
                j += 1;
            }

            while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                name.push(chars[j]);
                j += 1;
            }

            if !name.is_empty() {
                vars.push(name);
            }

            i = j;
            continue;
        }

        i += 1;
    }

    vars
}
