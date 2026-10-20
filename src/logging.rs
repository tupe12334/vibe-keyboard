use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use crate::presentation::tui::AppState;

struct TuiLogLayer {
    state: Arc<Mutex<AppState>>,
}

struct MessageVisitor(String);

impl Visit for MessageVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_string();
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }
}

impl<S: tracing::Subscriber> Layer<S> for TuiLogLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = MessageVisitor(String::new());
        event.record(&mut visitor);
        if visitor.0.is_empty() {
            return;
        }
        let msg = format!("[{}] {}", event.metadata().level(), visitor.0);
        if let Ok(mut s) = self.state.try_lock() {
            s.push_log(msg);
        }
    }
}

fn log_dir() -> std::path::PathBuf {
    crate::infrastructure::persistence::config_dir().join("vibe-keyboard")
}

/// Initialize the global tracing subscriber with a file appender and TUI layer.
/// The returned `WorkerGuard` must be kept alive for the duration of the program.
#[allow(clippy::print_stderr)]
pub fn init(state: Arc<Mutex<AppState>>) -> tracing_appender::non_blocking::WorkerGuard {
    let dir = log_dir();
    let _ = std::fs::create_dir_all(&dir);
    // Fall back to stderr when the log file cannot be created, instead of panicking.
    let sink: Box<dyn std::io::Write + Send> = match std::fs::File::create(dir.join("app.log")) {
        Ok(file) => Box::new(file),
        Err(e) => {
            eprintln!("cannot create app.log ({e}); logging to stderr");
            Box::new(std::io::stderr())
        }
    };
    let (non_blocking, guard) = tracing_appender::non_blocking(sink);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);

    let tui_layer = TuiLogLayer { state };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(tui_layer)
        .init();

    guard
}
