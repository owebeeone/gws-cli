#[cfg(test)]
use clap::CommandFactory;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Folds the operation event stream into the minimal state a single-line
/// progress display needs: how many members have started/finished and the
/// latest member activity to surface. Pure — terminal writing lives in
/// [`StderrProgressSink`], formatting in [`render_progress_line`].
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ProgressModel {
    pub(crate) label: String,
    pub(crate) started: usize,
    pub(crate) finished: usize,
    pub(crate) current_path: Option<String>,
    pub(crate) current_progress: Option<gwz_core::GitTransferProgress>,
}

impl ProgressModel {
    pub(crate) fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..Self::default()
        }
    }

    pub(crate) fn active(&self) -> usize {
        self.started.saturating_sub(self.finished)
    }

    /// Applies one event. Returns true when the display changed (so the sink
    /// can skip redrawing on events that do not affect the line).
    pub(crate) fn apply(&mut self, event: &gwz_core::OperationEvent) -> bool {
        use gwz_core::EventKind;
        match event.kind {
            EventKind::MemberStarted => {
                self.started += 1;
                self.current_path = event.member_path.clone();
                self.current_progress = None;
                true
            }
            EventKind::MemberProgress => {
                self.current_path = event.member_path.clone();
                self.current_progress = event.progress.clone();
                true
            }
            EventKind::MemberFinished => {
                self.finished += 1;
                if self.current_path == event.member_path {
                    self.current_path = None;
                    self.current_progress = None;
                }
                true
            }
            _ => false,
        }
    }
}

pub(crate) const SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Renders the model as one status line. `tick` advances the spinner so the
/// line shows liveness even when byte counts are momentarily static (e.g.
/// while resolving deltas).
pub(crate) fn render_progress_line(model: &ProgressModel, tick: usize) -> String {
    let spinner = SPINNER_FRAMES[tick % SPINNER_FRAMES.len()];
    let mut line = format!(
        "{spinner} {}: {} done, {} active",
        model.label,
        model.finished,
        model.active()
    );
    if let Some(path) = &model.current_path {
        line.push_str(" · ");
        line.push_str(member_short_name(path));
        if let Some(progress) = &model.current_progress {
            line.push(' ');
            line.push_str(progress_phase_label(progress.phase));
            let detail = progress_detail(progress);
            if !detail.is_empty() {
                line.push(' ');
                line.push_str(&detail);
            }
        }
    }
    line
}

pub(crate) fn progress_phase_label(phase: gwz_core::GitProgressPhase) -> &'static str {
    use gwz_core::GitProgressPhase as P;
    match phase {
        P::Enumerating => "enumerating",
        P::Counting => "counting",
        P::Compressing => "compressing",
        P::Receiving => "receiving",
        P::Resolving => "resolving",
        P::CheckingOut => "checking out",
        P::Writing => "writing",
    }
}

/// The detail tail for the current phase: "45% (1234/2730), 3.2 MiB" while
/// receiving, "78% (980/1254)" while resolving, a raw count while counting.
pub(crate) fn progress_detail(progress: &gwz_core::GitTransferProgress) -> String {
    use gwz_core::GitProgressPhase as P;
    match progress.phase {
        P::Receiving => {
            let mut parts = Vec::new();
            if let (Some(recv), Some(total)) = (progress.received_objects, progress.total_objects) {
                parts.push(format!("{}% ({recv}/{total})", pct(recv, total)));
            }
            if let Some(bytes) = progress.received_bytes {
                parts.push(human_bytes(bytes));
            }
            parts.join(", ")
        }
        P::Resolving => match (progress.indexed_deltas, progress.total_deltas) {
            (Some(idx), Some(total)) => format!("{}% ({idx}/{total})", pct(idx, total)),
            _ => String::new(),
        },
        P::Counting | P::Enumerating => progress
            .total_objects
            .or(progress.received_objects)
            .map(|n| n.to_string())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

/// Whole-percent of `n/d`, clamped to 0..=100.
pub(crate) fn pct(n: i64, d: i64) -> i64 {
    if d > 0 {
        (n.saturating_mul(100) / d).clamp(0, 100)
    } else {
        0
    }
}

/// Human-readable byte count in binary units (B/KiB/MiB/GiB).
pub(crate) fn human_bytes(bytes: i64) -> String {
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
    let bytes = bytes.max(0);
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Last path component for compact display, with a trailing `.git` stripped.
pub(crate) fn member_short_name(path: &str) -> &str {
    let trimmed = path.trim_end_matches('/');
    let name = trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed);
    name.strip_suffix(".git").unwrap_or(name)
}

/// Renders live progress to stderr as a single rewritten line while an
/// operation runs, then clears it. Active only when stderr is a terminal, so
/// piped or redirected runs stay clean; the machine-readable stream is
/// `--jsonl`. The model lock also serializes the concurrent member threads'
/// terminal writes.
pub(crate) struct StderrProgressSink {
    pub(crate) term: console::Term,
    pub(crate) enabled: bool,
    pub(crate) state: Mutex<ProgressModel>,
    pub(crate) tick: AtomicUsize,
}

impl StderrProgressSink {
    pub(crate) fn new(label: impl Into<String>) -> Self {
        let term = console::Term::stderr();
        let enabled = term.is_term();
        Self {
            term,
            enabled,
            state: Mutex::new(ProgressModel::new(label)),
            tick: AtomicUsize::new(0),
        }
    }
}

impl gwz_core::operation::EventSink for StderrProgressSink {
    fn deliver(&self, event: gwz_core::OperationEvent) {
        let mut state = self.state.lock().expect("progress state poisoned");
        let changed = state.apply(&event);
        if !self.enabled {
            return;
        }
        if event.kind == gwz_core::EventKind::OperationFinished {
            let _ = self.term.clear_line();
            return;
        }
        if !changed {
            return;
        }
        let tick = self.tick.fetch_add(1, Ordering::Relaxed);
        let line = truncate_to_width(&render_progress_line(&state, tick), &self.term);
        let _ = self.term.clear_line();
        let _ = self.term.write_str(&line);
    }
}

/// Truncates to one terminal width so the `\r` redraw never wraps and leaves
/// orphaned text. Width 0 (unknown) means no truncation.
pub(crate) fn truncate_to_width(line: &str, term: &console::Term) -> String {
    let width = term.size().1 as usize;
    if width == 0 || line.chars().count() <= width {
        return line.to_owned();
    }
    line.chars().take(width.saturating_sub(1)).collect()
}
