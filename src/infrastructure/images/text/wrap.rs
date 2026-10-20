pub(super) fn wrap(text: &str, per_line: usize, max_lines: usize) -> Vec<String> {
    let per_line = per_line.max(1);
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let joined = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if joined.chars().count() <= per_line {
            current = joined;
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        let mut rest: Vec<char> = word.chars().collect();
        while rest.len() > per_line {
            lines.push(rest.drain(..per_line).collect::<String>());
        }
        current = rest.into_iter().collect();
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines.truncate(max_lines);
    lines
}
