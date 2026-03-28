use std::fmt;

use nu_ansi_term::{Color, Style};
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields};
use tracing_subscriber::registry::LookupSpan;

struct MessageExtractor {
    message: String,
    fields: Vec<(String, String)>,
}

impl MessageExtractor {
    fn new() -> Self {
        Self {
            message: String::new(),
            fields: Vec::new(),
        }
    }
}

impl Visit for MessageExtractor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else {
            self.fields.push((field.name().to_string(), format!("{:?}", value)));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            self.fields.push((field.name().to_string(), value.to_string()));
        }
    }
}

pub struct ForgeFormatter;

/// Width to pad scope names to (longest scope name is "Dashboard")
const SCOPE_WIDTH: usize = 9;

const MODULE_COLORS: &[Color] = &[
    Color::Blue,
    Color::Cyan,
    Color::Green,
    Color::Purple,
    Color::Yellow,
    Color::Red,
];

fn module_color(name: &str) -> Color {
    let hash = name
        .bytes()
        .fold(0usize, |acc, b| acc.wrapping_add(b as usize));
    MODULE_COLORS[hash % MODULE_COLORS.len()]
}

/// Strip ANSI escape sequences from a string.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // skip until 'm' (end of ANSI escape)
            for c2 in chars.by_ref() {
                if c2 == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

impl<S, N> FormatEvent<S, N> for ForgeFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let ansi = writer.has_ansi_escapes();
        let level = *event.metadata().level();

        let level_color = match level {
            Level::ERROR => Color::Red,
            Level::WARN => Color::Yellow,
            Level::INFO => Color::Green,
            Level::DEBUG => Color::Blue,
            Level::TRACE => Color::Purple,
        };

        // level with color
        if ansi {
            write!(writer, "{} ", level_color.bold().paint(format!("{:>5}", level)))?;
        } else {
            write!(writer, "{:>5} ", level)?;
        }

        // module prefix from span fields
        let mut has_scope = false;
        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                if span.metadata().name() != "module" {
                    continue;
                }
                let ext = span.extensions();
                if let Some(fields) = ext.get::<FormattedFields<N>>() {
                    let raw = strip_ansi(&fields.to_string());
                    if let Some(val) = raw.strip_prefix("module=") {
                        let val = val.trim_matches('"');
                        has_scope = true;
                        if ansi {
                            let color = module_color(val);
                            let style = Style::new().fg(color).bold();
                            let padded = format!("{:>width$}", val, width = SCOPE_WIDTH);
                            write!(writer, "{} ", style.paint(padded))?;
                        } else {
                            write!(writer, "{:>width$} ", val, width = SCOPE_WIDTH)?;
                        }
                    }
                }
            }
        }
        if !has_scope {
            write!(writer, "{:>width$} ", "", width = SCOPE_WIDTH)?;
        }

        // extract message and fields separately
        let mut visitor = MessageExtractor::new();
        event.record(&mut visitor);

        if ansi {
            write!(writer, "{}", Style::new().fg(level_color).paint(&visitor.message))?;
            if !visitor.fields.is_empty() {
                let dimmed = Style::new().dimmed();
                for (key, val) in &visitor.fields {
                    write!(writer, " {}", dimmed.paint(format!("{}={}", key, val)))?;
                }
            }
        } else {
            write!(writer, "{}", visitor.message)?;
            for (key, val) in &visitor.fields {
                write!(writer, " {}={}", key, val)?;
            }
        }
        writeln!(writer)
    }
}
