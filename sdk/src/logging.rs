//! Logging setup for composition functions.

use tracing_subscriber::filter::{LevelFilter, ParseError, Targets};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// The gRPC, HTTP/2 and TLS crates under every function's server. Their
/// DEBUG output (h2 logs every frame) buries the function's own, so
/// `--debug` leaves them at INFO.
const TRANSPORT_TARGETS: [&str; 6] = ["h2", "hyper", "hyper_util", "tonic", "tower", "rustls"];

/// Configures process-wide logging to stderr. Call once, before serving.
///
/// Logs are JSON lines at info level. With debug enabled they are
/// human-readable, at debug level for everything but the gRPC transport
/// stack. `RUST_LOG`, when set, replaces the level selection in full. It
/// takes comma-separated `target=level` directives and bare levels, such as
/// `info,my_function=debug`; a `RUST_LOG` with span or field filters is
/// ignored with a warning. To hold back more crates under debug, use
/// [`Builder`].
pub fn configure(debug: bool) {
    Builder::new(debug).init();
}

/// Process-wide logging with options beyond [`configure`]. Create it with
/// [`Builder::new`], chain the options, then call [`Builder::init`]. New
/// options arrive as new methods, so code that builds logging keeps
/// compiling.
///
/// ```no_run
/// function_sdk_rust::logging::Builder::new(true)
///     .quiet(["wasmtime", "cranelift_codegen"])
///     .init();
/// ```
#[derive(Debug)]
pub struct Builder {
    debug: bool,
    quiet: Vec<String>,
}

impl Builder {
    /// Starts from [`configure`]'s defaults for the given `--debug` flag.
    pub fn new(debug: bool) -> Self {
        Self {
            debug,
            quiet: Vec::new(),
        }
    }

    /// Keeps these tracing targets (usually crate names, such as
    /// `wasmtime`) at INFO under debug, like the gRPC transport stack the
    /// SDK already holds back. A `RUST_LOG` that is set replaces the level
    /// selection, quiet targets included.
    pub fn quiet<I, S>(mut self, targets: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.quiet.extend(targets.into_iter().map(Into::into));
        self
    }

    /// Installs the global subscriber, writing to stderr. Call once,
    /// before serving.
    pub fn init(self) {
        let rust_log = std::env::var("RUST_LOG").ok();
        let (filter, ignored) = self.filter(rust_log.as_deref());
        // The fmt builder caps events at INFO by default; the filter decides.
        let builder = tracing_subscriber::fmt()
            .with_max_level(LevelFilter::TRACE)
            .with_writer(std::io::stderr)
            .with_file(true)
            .with_line_number(true);
        if self.debug {
            builder.finish().with(filter).init();
        } else {
            builder.json().finish().with(filter).init();
        }
        if let Some(error) = ignored {
            tracing::warn!(%error, "ignoring RUST_LOG, which takes only target=level directives");
        }
    }

    /// The level filter: `rust_log` when it is set, not blank and valid,
    /// else INFO everywhere - or, under debug, DEBUG everywhere except the
    /// transport stack and the quiet targets. Also returns why an invalid
    /// `rust_log` was ignored.
    fn filter(&self, rust_log: Option<&str>) -> (Targets, Option<ParseError>) {
        let ignored = match rust_log.map(str::trim).filter(|s| !s.is_empty()) {
            None => None,
            Some(set) => match set.parse() {
                Ok(filter) => return (filter, None),
                Err(error) => Some(error),
            },
        };
        if !self.debug {
            return (Targets::new().with_default(LevelFilter::INFO), ignored);
        }
        let filter = TRANSPORT_TARGETS
            .iter()
            .copied()
            .chain(self.quiet.iter().map(String::as_str))
            .fold(
                Targets::new().with_default(LevelFilter::DEBUG),
                |filter, target| filter.with_target(target, LevelFilter::INFO),
            );
        (filter, ignored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;

    fn enabled(builder: &Builder, rust_log: Option<&str>, target: &str, level: Level) -> bool {
        builder.filter(rust_log).0.would_enable(target, &level)
    }

    #[test]
    fn info_by_default() {
        let builder = Builder::new(false);
        for rust_log in [None, Some(" ")] {
            assert!(enabled(&builder, rust_log, "my_function", Level::INFO));
            assert!(!enabled(&builder, rust_log, "my_function", Level::DEBUG));
        }
    }

    #[test]
    fn debug_leaves_the_transport_at_info() {
        let builder = Builder::new(true);
        assert!(enabled(&builder, None, "my_function", Level::DEBUG));
        for target in TRANSPORT_TARGETS {
            let module = format!("{target}::proto");
            assert!(!enabled(&builder, None, &module, Level::DEBUG), "{target}");
            assert!(enabled(&builder, None, &module, Level::INFO), "{target}");
        }
    }

    #[test]
    fn quiet_targets_join_the_transport_under_debug() {
        let builder = Builder::new(true).quiet(["wasmtime", "cranelift_codegen"]);
        assert!(!enabled(&builder, None, "wasmtime::module", Level::DEBUG));
        assert!(!enabled(&builder, None, "cranelift_codegen", Level::DEBUG));
        assert!(enabled(&builder, None, "my_function", Level::DEBUG));
        let builder = Builder::new(false).quiet(["wasmtime"]);
        assert!(!enabled(&builder, None, "my_function", Level::DEBUG));
    }

    #[test]
    fn rust_log_replaces_the_flag_and_the_quiet_targets() {
        let builder = Builder::new(true).quiet(["wasmtime"]);
        let rust_log = Some("info,my_function=trace,wasmtime=debug");
        assert!(enabled(&builder, rust_log, "my_function", Level::TRACE));
        assert!(enabled(&builder, rust_log, "wasmtime", Level::DEBUG));
        assert!(!enabled(&builder, rust_log, "h2", Level::DEBUG));
        assert!(builder.filter(rust_log).1.is_none());
    }

    #[test]
    fn an_unsupported_rust_log_falls_back_to_the_flag() {
        let builder = Builder::new(false);
        let rust_log = Some("my_function[run{tag=x}]=debug");
        let (filter, ignored) = builder.filter(rust_log);
        assert!(ignored.is_some());
        assert!(filter.would_enable("my_function", &Level::INFO));
        assert!(!filter.would_enable("my_function", &Level::DEBUG));
    }
}
