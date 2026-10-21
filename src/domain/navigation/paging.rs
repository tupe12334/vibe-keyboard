use crate::domain::moadim::MoadimList;

/// Routines on one list page. Same page size as the other lists.
pub(super) const fn moadim_len(list: &MoadimList) -> usize {
    match list {
        MoadimList::Online(routines) => routines.len(),
        MoadimList::Pending | MoadimList::Offline => 0,
    }
}

/// Prompt keys on a session action page: keys 1 to 8. Eight prompts fit on one page.
pub const PROMPTS_PER_PAGE: usize = 8;

/// Key 9 on every session action page: FULL SCREEN, which focuses the session and toggles its window.
pub const FULL_SCREEN_KEY: u8 = 9;

/// Key 10 on every session action page: BLINK, which flashes the session's pane.
pub const BLINK_KEY: u8 = 10;

/// Number of action pages for `count` prompts. Always at least one.
pub const fn action_pages(count: usize) -> usize {
    if count == 0 {
        1
    } else {
        count.div_ceil(PROMPTS_PER_PAGE)
    }
}

/// True when key 15 is EXIT on `page`: the last action page. Earlier pages use key 15 for MORE.
pub const fn exit_on_page(page: usize, count: usize) -> bool {
    page + 1 >= action_pages(count)
}

/// Index into the prompt list for `key` on `page`. None for a key with no prompt on that page.
pub const fn prompt_index(page: usize, key: u8, count: usize) -> Option<usize> {
    if key < 1 || key as usize > PROMPTS_PER_PAGE {
        return None;
    }
    let index = page * PROMPTS_PER_PAGE + (key as usize - 1);
    if index < count {
        Some(index)
    } else {
        None
    }
}
