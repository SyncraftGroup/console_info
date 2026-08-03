use std::fmt::{self, Display};
use std::io::Write;

use inquire::Password;
use inquire::ui::Attributes;
use inquire::validator::StringValidator;
use owo_colors::OwoColorize;

pub use inquire::validator::Validation;

#[derive(Debug)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Locale {
    Zh,
    En,
}

pub fn get_locale() -> Locale {
    let locale = sys_locale::get_locale().unwrap_or_else(|| "en".into());
    if locale.starts_with("zh") {
        Locale::Zh
    } else {
        Locale::En
    }
}

/// Shorthand for a two-locale match.
///
/// ```ignore
/// msgs! { En: "Hello" Zh: "你好" },
/// ```
#[macro_export]
macro_rules! msgs {
    (En: $en:expr, Zh: $zh:expr $(,)?) => {{
        match $crate::get_locale() {
            $crate::Locale::En => ($en).to_string(),
            $crate::Locale::Zh => ($zh).to_string(),
        }
    }};
}

/// Format a reason+hint pair with a `→` arrow.
pub fn format_reason_hint(reason: &str, hint: &str) -> String {
    format!("{}\n  → {}", reason, hint)
}

/// Prompt the user for text input.
pub fn prompt_input(prompt: &str) -> Result<String, Error> {
    inquire::Text::new(prompt)
        .prompt()
        .map_err(|e| Error(e.to_string()))
}

/// Prompt the user for y/n confirmation.
pub fn prompt_confirm(prompt: &str) -> Result<bool, Error> {
    inquire::Confirm::new(prompt)
        .prompt()
        .map_err(|e| Error(e.to_string()))
}

/// Prompt the user to pick one option from a list (radio select).
pub fn prompt_select<T: Display>(prompt: &str, options: Vec<T>) -> Result<T, Error> {
    inquire::Select::new(prompt, options)
        .prompt()
        .map_err(|e| Error(e.to_string()))
}

enum BoxKind {
    Info,
    Warn,
    Error,
    Plain,
}

const L_STEP: &str = "[STEP]";
const L_OK: &str = "[ OK ]";
const L_INFO: &str = "[INFO]";
const L_WARN: &str = "[WARN]";
const L_FAIL: &str = "[FAIL]";
const L_QEST: &str = "[ ?? ]";
const L_GOT: &str = "[ -- ]";

/// Visual width of every label (used for border alignment).
const LABEL_W: usize = 6;

/// Stateless UI handle.  Obtain via [`Ui::get()`].
pub struct Ui;

impl Ui {
    pub fn get() -> Self {
        Ui
    }

    /// Print `[STEP]` in cyan.
    pub fn step(&self, content: &str) {
        eprintln!("{} {}", L_STEP.bright_cyan().bold(), content);
    }

    /// Print `[ OK ]` in green.
    pub fn ok(&self, content: &str) {
        eprintln!("{} {}", L_OK.bright_green().bold(), content);
    }

    /// Neutral information. Multi-line content is drawn in a dimmed box.
    pub fn info(&self, content: &str) {
        if content.contains('\n') {
            draw_box(content, BoxKind::Info);
        } else {
            eprintln!("{} {}", L_INFO.dimmed(), content);
        }
    }

    /// Non-fatal warning. Multi-line content is drawn in a yellow box.
    pub fn warn(&self, content: &str) {
        if content.contains('\n') {
            draw_box(content, BoxKind::Warn);
        } else {
            eprintln!("{} {}", L_WARN.yellow(), content);
        }
    }

    /// Error (does not exit). Multi-line content is drawn in a red box.
    pub fn error(&self, content: &str) {
        if content.contains('\n') {
            draw_box(content, BoxKind::Error);
        } else {
            eprintln!("{} {}", L_FAIL.red(), content);
        }
    }

    /// Plain text. Multi-line content gets a plain dimmed box (no label).
    pub fn plain(&self, content: &str) {
        if content.contains('\n') {
            draw_box(content, BoxKind::Plain);
        } else {
            eprintln!("{}", content);
        }
    }

    /// Prompt for text input.
    pub fn input(&self, content: &str) -> Result<String, Error> {
        prompt_input(&content)
    }

    /// Prompt for y/n confirmation.
    pub fn confirm(&self, content: &str) -> Result<bool, Error> {
        prompt_confirm(&content)
    }

    /// Prompt to pick one option from a list (radio select).
    pub fn select<T: Display>(&self, content: &str, options: Vec<T>) -> Result<T, Error> {
        prompt_select(&content, options)
    }

    pub fn echo(&self, text: &str) {
        eprintln!("{}", text);
    }

    pub fn password<V>(&self, content: &str, validator: V) -> Result<String, Error>
    where
        V: StringValidator + 'static,
    {
        Password::new(content)
            .with_validator(validator)
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .without_confirmation()
            .prompt()
            .map_err(|e| Error(e.to_string()))
    }
}

/// Pick a box width wide enough for content but bounded by the terminal.
fn box_width(content: &str) -> usize {
    let max_visual = content
        .lines()
        .map(|l| console::measure_text_width(l))
        .max()
        .unwrap_or(0);

    let term_w = console::Term::stderr().size().1 as usize;

    // Inner width = text + 2 spaces padding.  Outer adds 2 border chars.
    let inner_needed = max_visual + 2;
    let outer_needed = inner_needed + 2;

    outer_needed.clamp(44, term_w.saturating_sub(2).max(44))
}

/// Draw a labelled or plain Unicode box around `content`.
fn draw_box(content: &str, kind: BoxKind) {
    let w = box_width(content);
    let inner = w - 2;
    let text_area = inner - 2;

    let (top, bot, vert) = match kind {
        BoxKind::Plain => {
            let t = format!("┌{}┐", "─".repeat(w - 2));
            let b = format!("└{}┘", "─".repeat(w - 2));
            (
                t.dimmed().to_string(),
                b.dimmed().to_string(),
                "│".dimmed().to_string(),
            )
        }
        BoxKind::Info => {
            let t = format!(
                "┌─{}─{}┐",
                L_INFO,
                "─".repeat(w.saturating_sub(LABEL_W + 4))
            );
            let b = format!("└{}┘", "─".repeat(w - 2));
            (
                t.blue().to_string(),
                b.blue().to_string(),
                "│".blue().to_string(),
            )
        }
        BoxKind::Warn => {
            let t = format!(
                "┌─{}─{}┐",
                L_WARN,
                "─".repeat(w.saturating_sub(LABEL_W + 4))
            );
            let b = format!("└{}┘", "─".repeat(w - 2));
            (
                t.yellow().to_string(),
                b.yellow().to_string(),
                "│".yellow().to_string(),
            )
        }
        BoxKind::Error => {
            let t = format!(
                "┌─{}─{}┐",
                L_FAIL,
                "─".repeat(w.saturating_sub(LABEL_W + 4))
            );
            let b = format!("└{}┘", "─".repeat(w - 2));
            (
                t.red().to_string(),
                b.red().to_string(),
                "│".red().to_string(),
            )
        }
    };

    let mut stderr = std::io::stderr().lock();
    let _ = writeln!(stderr, "{top}");
    for line in content.lines() {
        let vis = console::measure_text_width(line);
        let rhs_pad = text_area.saturating_sub(vis);
        let _ = writeln!(stderr, "{vert} {line}{} {vert}", " ".repeat(rhs_pad));
    }
    let _ = writeln!(stderr, "{bot}");
}

/// Configure inquire's prompt colours to match the Syncraft theme.
/// Call once at startup.
pub fn configure_inquire() {
    use inquire::ui::{Color, RenderConfig, Styled};

    let config = RenderConfig::default()
        .with_prompt_prefix(
            Styled::new(L_QEST)
                .with_fg(Color::LightBlue)
                .with_attr(Attributes::BOLD),
        )
        .with_answered_prompt_prefix(
            Styled::new(L_GOT)
                .with_fg(Color::LightBlue)
                .with_attr(Attributes::BOLD),
        );

    inquire::set_global_render_config(config);
}
