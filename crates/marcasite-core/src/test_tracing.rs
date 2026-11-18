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

/// A layer installed once as the process-wide default subscriber. It records into the
/// buffer of the current thread's [`capture`] call, if any.
///
/// A global subscriber, rather than one `set_default` per test, avoids a race in tracing's
/// global callsite-interest cache: with per-thread subscribers, a callsite first hit on a
/// thread without one could be cached as "never" and a capturing test would miss its span.
struct Capture;

thread_local! {
    static CURRENT: std::cell::RefCell<Option<Arc<Mutex<Captured>>>> =
        const { std::cell::RefCell::new(None) };
}

fn current() -> Option<Arc<Mutex<Captured>>> {
    CURRENT.with(|c| c.borrow().clone())
}

/// Where a span's fields live: its capture buffer and index, stored in the span's extensions.
struct SpanIndex(Arc<Mutex<Captured>>, usize);

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Capture {
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let Some(buffer) = current() else { return };
        let mut fields = Fields::default();
        attrs.record(&mut fields);
        let index = {
            let mut captured = buffer.lock().unwrap();
            captured
                .spans
                .push((attrs.metadata().name().to_owned(), fields.0));
            captured.spans.len() - 1
        };
        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(SpanIndex(buffer, index));
        }
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        values.record(&mut fields);
        let Some(span) = ctx.span(id) else { return };
        let extensions = span.extensions();
        let Some(SpanIndex(buffer, index)) = extensions.get::<SpanIndex>() else {
            return;
        };
        buffer.lock().unwrap().spans[*index].1.extend(fields.0);
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let Some(buffer) = current() else { return };
        let mut fields = Fields::default();
        event.record(&mut fields);
        let message = fields.0.remove("message").unwrap_or_default();
        let scope = ctx
            .event_scope(event)
            .map(|scope| scope.map(|span| span.name().to_owned()).collect())
            .unwrap_or_default();
        buffer.lock().unwrap().events.push((message, scope));
    }
}

/// Clears the current thread's capture buffer when dropped.
pub(crate) struct CaptureGuard(());

impl Drop for CaptureGuard {
    fn drop(&mut self) {
        CURRENT.with(|c| *c.borrow_mut() = None);
    }
}

/// Captures spans and events created on the current thread until the guard is dropped.
///
/// Use with a current-thread runtime (the `#[tokio::test]` default) so spawned tasks run on
/// the thread that is capturing.
pub(crate) fn capture() -> (Arc<Mutex<Captured>>, CaptureGuard) {
    static INSTALL: std::sync::Once = std::sync::Once::new();
    INSTALL.call_once(|| {
        // Fails only if another global subscriber was installed first; tests install none.
        let _ =
            tracing::subscriber::set_global_default(tracing_subscriber::registry().with(Capture));
    });
    // A callsite another thread registered while the subscriber was being installed can keep a
    // stale "never" interest; recompute every callsite's interest so none is missed.
    tracing::callsite::rebuild_interest_cache();
    let captured = Arc::new(Mutex::new(Captured::default()));
    CURRENT.with(|c| *c.borrow_mut() = Some(captured.clone()));
    (captured, CaptureGuard(()))
}
