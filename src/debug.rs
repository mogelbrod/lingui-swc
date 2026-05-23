use std::cell::RefCell;
use std::rc::Rc;

use swc_core::common::errors::SourceMapper;
use swc_core::common::sync::Lrc;
use swc_core::common::{SourceMap, Span};
use swc_core::plugin::proxies::PluginSourceMapProxy;

thread_local! {
    static DEBUG_LOG_CAPTURE: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
}

#[derive(Clone)]
pub enum DebugSourceMap {
    Plugin(PluginSourceMapProxy),
    Test(Lrc<SourceMap>),
}

impl DebugSourceMap {
    pub fn from_plugin(source_map: PluginSourceMapProxy) -> Self {
        Self::Plugin(source_map)
    }

    pub fn from_test(source_map: Lrc<SourceMap>) -> Self {
        Self::Test(source_map)
    }

    fn span_to_filename(&self, span: Span) -> String {
        match self {
            Self::Plugin(source_map) => source_map.span_to_filename(span).to_string(),
            Self::Test(source_map) => source_map.span_to_filename(span).to_string(),
        }
    }

    fn line_for_span(&self, span: Span) -> usize {
        match self {
            Self::Plugin(source_map) => source_map.lookup_char_pos(span.lo).line,
            Self::Test(source_map) => source_map.lookup_char_pos(span.lo).line,
        }
    }
}

#[derive(Clone)]
pub struct DebugLogBuffer(Rc<RefCell<Vec<DebugLogEvent>>>);

#[derive(Clone)]
struct DebugLogEvent {
    pos: u32,
    message: String,
}

impl DebugLogBuffer {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(vec![])))
    }

    fn push(&self, pos: u32, message: String) {
        self.0.borrow_mut().push(DebugLogEvent { pos, message });
    }

    pub fn flush(&self) {
        let mut events = self.0.borrow_mut();
        events.sort_by_key(|event| event.pos);

        for event in events.drain(..) {
            let was_captured = DEBUG_LOG_CAPTURE.with(|capture| {
                let mut capture = capture.borrow_mut();
                if let Some(lines) = capture.as_mut() {
                    lines.push(event.message.clone());
                    true
                } else {
                    false
                }
            });

            if !was_captured {
                eprintln!("{}", event.message);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct DebugSettings<'a> {
    pub enabled: bool,
    pub source_map: &'a DebugSourceMap,
    pub buffer: &'a DebugLogBuffer,
}

impl DebugSettings<'_> {
    pub fn log(&self, span: Span, name: &str, attrs: Vec<(&'static str, String)>) {
        if !self.enabled || span.is_dummy() {
            return;
        }

        let file_name = self.source_map.span_to_filename(span);
        let line_number = self.source_map.line_for_span(span);
        let attributes = format_attributes(attrs);

        let message = if attributes.is_empty() {
            format!("{file_name}:{line_number}: {name}")
        } else {
            format!("{file_name}:{line_number}: {name} {attributes}")
        };

        self.buffer.push(span.lo.0, message);
    }
}

pub fn capture_debug_logs<T>(callback: impl FnOnce() -> T) -> (T, Vec<String>) {
    DEBUG_LOG_CAPTURE.with(|capture| {
        let previous = capture.replace(Some(vec![]));
        let result = callback();
        let logs = capture.replace(previous).unwrap_or_default();
        (result, logs)
    })
}

fn format_attributes(attrs: Vec<(&'static str, String)>) -> String {
    attrs
        .into_iter()
        .map(|(key, value)| format!(r#"{key}="{}""#, escape_attr_value(&value)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn escape_attr_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', r#"\""#)
}
