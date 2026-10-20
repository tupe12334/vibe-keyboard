use super::*;

// ps -Ao pid=,ppid=,tty=,comm= : a claude under the VS Code terminal (ttys001), one in Terminal.app
// (ttys002), one under login with no VS Code ancestor (ttys003), and a stray line.
const PS: &str = "  100 1 ?? /Applications/Visual Studio Code.app/Contents/MacOS/Electron\n\
                  \x20 110 100 ?? /Applications/Visual Studio Code.app/Contents/Frameworks/Code Helper (Plugin).app/Contents/MacOS/Code Helper (Plugin)\n\
                  \x20 120 110 ttys001 -zsh\n\
                  \x20 121 120 ttys001 /Users/me/.local/bin/claude\n\
                  \x20 200 1 ?? /System/Applications/Utilities/Terminal.app/Contents/MacOS/Terminal\n\
                  \x20 210 200 ttys002 -zsh\n\
                  \x20 211 210 ttys002 claude\n\
                  \x20 300 1 ttys003 login\n\
                  \x20 301 300 ttys003 claude\n\
                  \x20 bad line\n";

#[test]
fn vscode_session_needs_a_vscode_ancestor() {
    assert!(is_vscode_session(PS, "ttys001"));
    assert!(!is_vscode_session(PS, "ttys002"), "Terminal.app tab");
    assert!(!is_vscode_session(PS, "ttys003"), "no VS Code ancestor");
    assert!(!is_vscode_session(PS, "ttys009"), "no claude on that tty");
    assert!(!is_vscode_session("", "ttys001"));
}

#[test]
fn vscode_check_ignores_a_claude_named_only_in_a_parent_path() {
    let ps = "  10 1 ttys004 /Applications/Claude Code/claude\n";
    assert!(!is_vscode_session(ps, "ttys004"));
}
