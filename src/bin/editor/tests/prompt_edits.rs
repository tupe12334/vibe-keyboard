use super::*;

#[test]
fn prompt_ops_reorder_delete_and_add() {
    let prompt = |label: &str| Prompt {
        label: label.into(),
        text: format!("/{label}"),
    };
    let mut prompts = vec![prompt("a"), prompt("b"), prompt("c")];
    apply_prompt_op(&mut prompts, PromptOp::Down(0));
    assert_eq!(
        prompts.iter().map(|p| p.label.as_str()).collect::<Vec<_>>(),
        ["b", "a", "c"]
    );
    apply_prompt_op(&mut prompts, PromptOp::Up(0));
    assert_eq!(prompts[0].label, "b", "Up on the first row does nothing");
    apply_prompt_op(&mut prompts, PromptOp::Delete(1));
    assert_eq!(
        prompts.iter().map(|p| p.label.as_str()).collect::<Vec<_>>(),
        ["b", "c"]
    );
    apply_prompt_op(&mut prompts, PromptOp::Delete(9));
    apply_prompt_op(&mut prompts, PromptOp::Add);
    assert_eq!(prompts.len(), 3);
    assert_eq!(prompts[2].text, "");
}
