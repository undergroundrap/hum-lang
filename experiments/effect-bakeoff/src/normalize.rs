use std::collections::BTreeMap;

#[derive(Default)]
pub struct AlphaNormalizer {
    variables: BTreeMap<String, String>,
}

impl AlphaNormalizer {
    pub fn normalize(&mut self, input: &str) -> String {
        let chars: Vec<char> = input.chars().collect();
        let mut output = String::new();
        let mut index = 0;
        while index < chars.len() {
            if chars[index] == '$' && index + 1 < chars.len() && is_ident_start(chars[index + 1]) {
                let start = index;
                index += 2;
                while index < chars.len() && is_ident_continue(chars[index]) {
                    index += 1;
                }
                let original: String = chars[start..index].iter().collect();
                let next = self.variables.len();
                let normalized = self
                    .variables
                    .entry(original)
                    .or_insert_with(|| format!("$v{next}"));
                output.push_str(normalized);
            } else {
                output.push(chars[index]);
                index += 1;
            }
        }
        output
    }
}

fn is_ident_start(value: char) -> bool {
    value == '_' || value.is_ascii_alphabetic()
}

fn is_ident_continue(value: char) -> bool {
    value == '_' || value.is_ascii_alphanumeric()
}

pub fn normalize_identity_path(path: &str) -> String {
    path.replace('\\', "/")
}

pub fn push_field(output: &mut String, name: &str, value: &str) {
    output.push_str(&name.len().to_string());
    output.push(':');
    output.push_str(name);
    output.push('=');
    output.push_str(&value.len().to_string());
    output.push(':');
    output.push_str(value);
    output.push('\n');
}
