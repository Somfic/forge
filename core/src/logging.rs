use std::fmt;

use nu_ansi_term::{Color, Style};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields};
use tracing_subscriber::registry::LookupSpan;

pub struct ForgeFormatter;

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

        // level with color
        if ansi {
            let color = match level {
                Level::ERROR => Color::Red,
                Level::WARN => Color::Yellow,
                Level::INFO => Color::Green,
                Level::DEBUG => Color::Blue,
                Level::TRACE => Color::Purple,
            };
            write!(writer, "{} ", color.bold().paint(format!("{:>5}", level)))?;
        } else {
            write!(writer, "{:>5} ", level)?;
        }

        // module prefix from span fields
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
                        if ansi {
                            let color = module_color(val);
                            let style = Style::new().fg(color).bold();
                            write!(writer, "{} ", style.paint(val))?;
                        } else {
                            write!(writer, "{}: ", val)?;
                        }
                    }
                }
            }
        }

        // event message
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}
