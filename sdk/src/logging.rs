//! Logging setup for composition functions.

use tracing_subscriber::EnvFilter;

/// The gRPC, HTTP/2 and TLS crates under every function's server. Their
/// DEBUG output (h2 logs every frame) buries the function's own, so
/// `--debug` leaves them at INFO.
const TRANSPORT_TARGETS: [&str; 6] = ["h2", "hyper", "hyper_util", "tonic", "tower", "rustls"];

/// Configures process-wide logging to stderr. Call once, before serving.
///
/// Logs are JSON lines at info level. With debug enabled they are
/// human-readable, at debug level for everything but the gRPC transport
/// stack. `RUST_LOG`, when set, replaces the level selection in full. To
/// hold back more crates under debug, use [`Builder`].
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
        // parse_lossy reports a malformed directive on stderr and applies the rest.
        let filter = EnvFilter::builder().parse_lossy(self.directives(rust_log.as_deref()));
        let builder = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::stderr)
            .with_file(true)
            .with_line_number(true);
        if self.debug {
            builder.init();
        } else {
            builder.json().init();
        }
    }

    /// The filter directives: `rust_log` verbatim when it is set and not
    /// blank, else INFO everywhere - or, under debug, DEBUG everywhere
    /// except the transport stack and the quiet targets.
    fn directives(&self, rust_log: Option<&str>) -> String {
        if let Some(set) = rust_log.map(str::trim).filter(|s| !s.is_empty()) {
            return set.to_string();
        }
        if !self.debug {
            return "info".to_string();
        }
        let mut directives = String::from("debug");
        for target in TRANSPORT_TARGETS
            .iter()
            .copied()
            .chain(self.quiet.iter().map(String::as_str))
        {
            directives.push_str(&format!(",{target}=info"));
        }
        directives
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRANSPORT_AT_INFO: &str =
        "debug,h2=info,hyper=info,hyper_util=info,tonic=info,tower=info,rustls=info";

    #[test]
    fn info_by_default() {
        assert_eq!(Builder::new(false).directives(None), "info");
        assert_eq!(Builder::new(false).directives(Some(" ")), "info");
    }

    #[test]
    fn debug_leaves_the_transport_at_info() {
        assert_eq!(Builder::new(true).directives(None), TRANSPORT_AT_INFO);
        EnvFilter::try_new(Builder::new(true).directives(None)).expect("valid directives");
    }

    #[test]
    fn quiet_targets_join_the_transport_under_debug() {
        let builder = Builder::new(true).quiet(["wasmtime", "cranelift_codegen"]);
        assert_eq!(
            builder.directives(None),
            format!("{TRANSPORT_AT_INFO},wasmtime=info,cranelift_codegen=info")
        );
        EnvFilter::try_new(builder.directives(None)).expect("valid directives");
        assert_eq!(
            Builder::new(false).quiet(["wasmtime"]).directives(None),
            "info"
        );
    }

    #[test]
    fn rust_log_replaces_the_flag_and_the_quiet_targets() {
        assert_eq!(
            Builder::new(true)
                .quiet(["wasmtime"])
                .directives(Some("info,my_function=trace")),
            "info,my_function=trace"
        );
    }
}
