//! Edits to the session prompt list, shared by every editor. Rows are addressed by index.
//! An out-of-range row does nothing.
use super::profile::Prompt;

/// The row a new prompt starts as. Its empty text fails the save check until the user fills it in.
pub fn new_prompt() -> Prompt {
    Prompt {
        label: "New".into(),
        text: String::new(),
    }
}

/// Add a new row at the end.
pub fn add_prompt(prompts: &mut Vec<Prompt>) {
    prompts.push(new_prompt());
}

/// Swap row `index` with the row above (`down` false) or below (`down` true).
pub const fn move_prompt(prompts: &mut [Prompt], index: usize, down: bool) {
    let Some(target) = (if down {
        index.checked_add(1)
    } else {
        index.checked_sub(1)
    }) else {
        return;
    };
    if index < prompts.len() && target < prompts.len() {
        prompts.swap(index, target);
    }
}

/// Remove row `index`.
pub fn delete_prompt(prompts: &mut Vec<Prompt>, index: usize) {
    if index < prompts.len() {
        prompts.remove(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(prompts: &[Prompt]) -> Vec<&str> {
        prompts.iter().map(|p| p.label.as_str()).collect()
    }

    #[test]
    fn rows_move_delete_and_add_by_index() {
        let row = |label: &str| Prompt {
            label: label.into(),
            text: format!("/{label}"),
        };
        let mut prompts = vec![row("a"), row("b"), row("c")];
        move_prompt(&mut prompts, 0, true);
        assert_eq!(labels(&prompts), ["b", "a", "c"]);
        move_prompt(&mut prompts, 0, false);
        assert_eq!(
            labels(&prompts),
            ["b", "a", "c"],
            "up on the first row does nothing"
        );
        move_prompt(&mut prompts, 2, true);
        assert_eq!(
            labels(&prompts),
            ["b", "a", "c"],
            "down on the last row does nothing"
        );
        delete_prompt(&mut prompts, 1);
        assert_eq!(labels(&prompts), ["b", "c"]);
        delete_prompt(&mut prompts, 9);
        assert_eq!(labels(&prompts), ["b", "c"]);
        add_prompt(&mut prompts);
        assert_eq!(labels(&prompts), ["b", "c", "New"]);
        assert_eq!(prompts[2].text, "", "a new row has empty text");
    }
}
