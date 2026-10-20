//! Exit path: report `Stopped` whenever the loop gives up, so the caller always hears it.
use super::event::ControllerEvent;

/// Pass `result` through. When it is None the loop is leaving, so report `Stopped` first.
pub fn stop_if_none<T>(result: Option<T>, events: &dyn Fn(ControllerEvent)) -> Option<T> {
    if result.is_none() {
        events(ControllerEvent::Stopped);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn reports_stopped_only_when_result_is_none() {
        let seen = RefCell::new(Vec::new());
        let sink = |e: ControllerEvent| seen.borrow_mut().push(e);
        assert_eq!(stop_if_none(Some(7u8), &sink), Some(7));
        assert!(seen.borrow().is_empty());
        assert_eq!(stop_if_none::<u8>(None, &sink), None);
        assert_eq!(*seen.borrow(), vec![ControllerEvent::Stopped]);
    }
}
