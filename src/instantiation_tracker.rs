use clang::Entity;

use crate::types::TemplateInstantiation;

pub fn extract_template_instantiation(entity: &Entity<'_>) -> Option<TemplateInstantiation> {
    let display_name = entity
        .get_display_name()
        .or_else(|| entity.get_name())
        .unwrap_or_default();
    parse_template_instantiation(&display_name)
}

pub fn parse_template_instantiation(name: &str) -> Option<TemplateInstantiation> {
    let start = name.find('<')?;
    let end = name.rfind('>')?;
    if end <= start {
        return None;
    }

    let template_name = name[..start].trim().to_string();
    if template_name.is_empty() {
        return None;
    }

    let args = split_template_args(&name[start + 1..end]);
    let sanitized_args = args
        .iter()
        .map(|arg| sanitize_identifier(arg))
        .collect::<Vec<_>>();
    let instantiated_name = format!(
        "{}_{}",
        sanitize_identifier(&template_name),
        sanitized_args.join("_")
    );

    Some(TemplateInstantiation {
        template_name,
        type_args: args,
        instantiated_name,
    })
}

fn split_template_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut depth = 0usize;
    let mut current = String::new();

    for ch in input.chars() {
        match ch {
            '<' => {
                depth += 1;
                current.push(ch);
            }
            '>' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 => {
                if !current.trim().is_empty() {
                    args.push(current.trim().to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if !current.trim().is_empty() {
        args.push(current.trim().to_string());
    }

    args
}

fn sanitize_identifier(input: &str) -> String {
    input
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}
