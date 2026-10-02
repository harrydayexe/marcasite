//! A tracing layer that captures spans and events for assertions in tests.

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Id, Record},
};
use tracing_subscriber::{Layer, layer::Context, prelude::*, registry::LookupSpan};

/// What a [`Capture`] layer saw.
#[derive(Debug, Default)]
pub(crate) struct Captured {
    /// Every span ever created, by name, with its fields (including later `record`s).
    pub(crate) spans: Vec<(String, HashMap<String, String>)>,
    /// Every event's message, with the names of its enclosing spans (innermost first).
    pub(crate) events: Vec<(String, Vec<String>)>,
}

impl Captured {
    /// The fields of the last span named `name`.
    pub(crate) fn span(&self, name: &str) -> Option<&HashMap<String, String>> {
        self.spans
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, f)| f)
    }

    /// The enclosing span names of the first event whose message contains `message`.
    pub(crate) fn event_scope(&self, message: &str) -> Option<&[String]> {
        self.events
            .iter()
            .find(|(m, _)| m.contains(message))
            .map(|(_, scope)| scope.as_slice())
    }
}

#[derive(Default)]
struct Fields(HashMap<String, String>);

impl Visit for Fields {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.0.insert(field.name().to_owned(), format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_owned(), value.to_owned());
    }
}

struct Capture(Arc<Mutex<Captured>>);

/// Index of a span in `Captured::spans`, stored in the span's extensions.
struct SpanIndex(usize);

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Capture {
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        attrs.record(&mut fields);
        let mut captured = self.0.lock().unwrap();
        captured
            .spans
            .push((attrs.metadata().name().to_owned(), fields.0));
        let index = captured.spans.len() - 1;
        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(SpanIndex(index));
        }
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        values.record(&mut fields);
        let Some(span) = ctx.span(id) else { return };
        let extensions = span.extensions();
        let Some(SpanIndex(index)) = extensions.get::<SpanIndex>() else {
            return;
        };
        self.0.lock().unwrap().spans[*index].1.extend(fields.0);
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        event.record(&mut fields);
        let message = fields.0.remove("message").unwrap_or_default();
        let scope = ctx
            .event_scope(event)
            .map(|scope| scope.map(|span| span.name().to_owned()).collect())
            .unwrap_or_default();
        self.0.lock().unwrap().events.push((message, scope));
    }
}

/// Installs a capturing subscriber on the current thread until the guard is dropped.
///
/// Use with a current-thread runtime (the `#[tokio::test]` default) so spawned tasks run on
/// the thread that has the subscriber.
pub(crate) fn capture() -> (Arc<Mutex<Captured>>, tracing::subscriber::DefaultGuard) {
    let captured = Arc::new(Mutex::new(Captured::default()));
    let subscriber = tracing_subscriber::registry().with(Capture(captured.clone()));
    let guard = tracing::subscriber::set_default(subscriber);
    (captured, guard)
}
