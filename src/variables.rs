use crate::error::ExpandError;
use std::collections::{HashMap, HashSet};

const MAX_EXPRESSION_DEPTH: u32 = 32;

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
                        if var_val.map_or(true, |v| v.is_empty()) {
                            val
                        } else {
                            var_val.unwrap().clone()
                        }
                    } else if var_val.map_or(false, |v| !v.is_empty()) {
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
                    .ok_or_else(|| ExpandError::VariableNotFound {
                        name: name.clone(),
                    })?;
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
