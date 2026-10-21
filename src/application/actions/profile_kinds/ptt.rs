//! Push-to-talk: a tap posts one key. A hold reserves it, starts it and ends it.

use super::{calls, Rig};
use crate::domain::hotkey::{default_push_to_talk, push_to_talk_key};
use crate::domain::profile::Action;

pub fn push_to_talk() {
    let space = format!("{:?}", default_push_to_talk());
    {
        let mut tap = Rig::new(Action::PushToTalk { key: None }, None);
        tap.press();
    }
    assert_eq!(calls(), [format!("tap {space}")]);

    {
        let mut hold = Rig::new(Action::None, Some(Action::PushToTalk { key: None }));
        hold.press();
        hold.poll();
        hold.release();
    }
    assert_eq!(
        calls(),
        [
            "hold_begin 1".to_string(),
            format!("ptt_start gen=7 key=1 {space} tty=None"),
            "hold_end 1".to_string(),
        ]
    );

    let Ok(bound_hotkey) = push_to_talk_key(Some("cmd+k")) else {
        panic!("cmd+k must parse as a push-to-talk key");
    };
    {
        let mut bound = Rig::new(
            Action::PushToTalk {
                key: Some("cmd+k".into()),
            },
            None,
        );
        bound.press();
    }
    assert_eq!(calls(), [format!("tap {bound_hotkey:?}")]);
}
