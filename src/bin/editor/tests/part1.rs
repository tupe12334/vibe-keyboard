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

#[test]
fn discard_is_refused_when_clean() {
    let mut session = Session::new(Profile::default_from_legacy());
    let now = Instant::now();
    assert!(!session.dirty());
    assert!(!session.discard(Profile::default_from_legacy(), now));
    assert!(!session.undo_available(now));
}

#[test]
fn dirty_then_discard_then_undo_returns_to_dirty() {
    let disk = Profile::default_from_legacy();
    let mut session = Session::new(disk.clone());
    session.profile = edited_layout();
    let now = Instant::now();
    assert!(session.dirty());

    assert!(session.discard(disk.clone(), now));
    assert!(!session.dirty());
    assert_eq!(session.profile, disk);
    assert!(session.undo_available(now));

    assert!(session.undo(now));
    assert!(session.dirty());
    assert_eq!(session.profile, edited_layout());
    assert!(!session.undo_available(now));
}

#[test]
fn undo_expires_after_the_window() {
    let disk = Profile::default_from_legacy();
    let mut session = Session::new(disk.clone());
    session.profile = edited_layout();
    let start = Instant::now();
    assert!(session.discard(disk.clone(), start));
    let later = start + UNDO_WINDOW + Duration::from_millis(1);
    assert!(!session.undo_available(later));
    session.expire(later);
    assert!(!session.undo(later));
    assert_eq!(session.profile, disk);
}

#[test]
fn save_clears_the_undo() {
    let disk = Profile::default_from_legacy();
    let mut session = Session::new(disk.clone());
    session.profile = edited_layout();
    let now = Instant::now();
    assert!(session.discard(disk, now));
    session.mark_saved();
    assert!(!session.undo_available(now));
}
