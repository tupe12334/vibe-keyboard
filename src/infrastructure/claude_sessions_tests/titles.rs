use super::*;

#[test]
fn tab_titles_need_a_valid_device_path() {
    let out = "/dev/ttys003\t✳ Alpha (claude)\n/dev/ttys00x\tbad\nttys004\tno device\n\
               /dev/ttys005\t\n";
    let titles = parse_tab_titles(out);
    assert_eq!(
        titles.get("ttys003").map(String::as_str),
        Some("✳ Alpha (claude)")
    );
    assert_eq!(titles.get("ttys005").map(String::as_str), Some(""));
    assert_eq!(titles.len(), 2);
}

#[test]
fn summary_is_the_last_summary_else_the_first_prompt() {
    let text = concat!(
        "not json\n",
        r#"{"type":"user","message":{"content":"Fix the login bug please, it fails"}}"#,
        "\n",
        r#"{"type":"summary","summary":"Old name"}"#,
        "\n",
        r#"{"type":"summary","summary":"Login fix"}"#,
        "\n",
    );
    assert_eq!(summary_from_jsonl(text), Some("Login fix".to_string()));
    let prompt_only = concat!(
        r#"{"type":"user","message":{"content":[{"type":"text","text":"Build the thing"}]}}"#,
        "\n",
        r#"{"type":"user","message":{"content":"second prompt"}}"#,
        "\n",
    );
    assert_eq!(
        summary_from_jsonl(prompt_only),
        Some("Build the thing".to_string())
    );
    let long = format!(
        r#"{{"type":"user","message":{{"content":"{}"}}}}"#,
        "x".repeat(200)
    );
    assert_eq!(
        summary_from_jsonl(&long).map(|s| s.chars().count()),
        Some(SUMMARY_CHARS)
    );
}

#[test]
fn real_tab_line_gives_the_title_and_url_prompts_are_skipped() {
    let out = "/dev/ttys000\t✳ Centy availability (claude)\n";
    let titles = parse_tab_titles(out);
    assert_eq!(
        titles.get("ttys000").map(String::as_str),
        Some("✳ Centy availability (claude)")
    );
    let url_first = concat!(
        r#"{"type":"user","message":{"content":"https://example.com/pasted"}}"#,
        "\n",
        r#"{"type":"user","message":{"content":"Write the report"}}"#,
        "\n",
    );
    assert_eq!(
        summary_from_jsonl(url_first),
        Some("Write the report".to_string())
    );
    let url_only = r#"{"type":"user","message":{"content":"http://x.test"}}"#;
    assert_eq!(summary_from_jsonl(url_only), None);
}

#[test]
fn user_line_without_text_gives_no_summary() {
    let tool_only = r#"{"type":"user","message":{"content":[{"type":"tool_result"}]}}"#;
    let numeric = r#"{"type":"user","message":{"content":42}}"#;
    assert_eq!(summary_from_jsonl(tool_only), None);
    assert_eq!(summary_from_jsonl(numeric), None);
    assert_eq!(summary_from_jsonl(r#"{"type":"assistant"}"#), None);
}
