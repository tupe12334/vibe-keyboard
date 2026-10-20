use crate::domain::moadim::MoadimList;

/// Routines on one list page. Same page size as the other lists.
pub(super) const fn moadim_len(list: &MoadimList) -> usize {
    match list {
        MoadimList::Online(routines) => routines.len(),
        MoadimList::Pending | MoadimList::Offline => 0,
    }
}

/// Prompt keys on a session action page: keys 1 to 10. Ten prompts fit on one page.
pub const PROMPTS_PER_PAGE: usize = 10;

/// Number of action pages for `count` prompts. Always at least one.
pub const fn action_pages(count: usize) -> usize {
    if count == 0 {
        1
    } else {
        count.div_ceil(PROMPTS_PER_PAGE)
    }
}

/// Index into the prompt list for `key` on `page`. None for a key with no prompt on that page.
pub const fn prompt_index(page: usize, key: u8, count: usize) -> Option<usize> {
    if key < 1 || key > 10 {
        return None;
    }
    let index = page * PROMPTS_PER_PAGE + (key as usize - 1);
    if index < count {
        Some(index)
    } else {
        None
    }
}
