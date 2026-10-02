//! Turn paginated endpoints into [`Stream`]s of items.
//!
//! Service clients expose each paginated endpoint twice:
//!
//! - `send()` fetches a single page (raw page access, including the cursor), and
//! - `into_stream()` walks every page lazily and yields individual items, as a
//!   [`Paginated`] stream.
//!
//! The helpers here implement the walking for the two pagination styles the APIs use:
//! opaque cursors ([`cursor_stream`]) and numeric offsets ([`offset_stream`]).
//!
//! Streams stop after the first error (which is yielded). Use
//! [`futures_util::StreamExt::take`] to bound the number of items, and
//! [`futures_util::TryStreamExt::try_collect`] to gather them.
//!
//! [`Stream`]: futures_core::Stream

use std::{
    collections::VecDeque,
    fmt,
    future::Future,
    pin::Pin,
    sync::{Mutex, PoisonError},
    task::{Context, Poll, ready},
};

use futures_core::{FusedStream, Stream};

use crate::Result;

/// The items of a paginated endpoint, fetched page by page as the stream is polled.
///
/// Every `into_stream()` returns this type. It yields `Ok(item)` for each item, in order,
/// and ends after the last page, or right after yielding the first error. It is
/// [`Unpin`], so it can be polled directly in a `while let` loop, and it is a named type,
/// so it can be stored in a struct. It is `Send + Sync + 'static`, and it implements
/// [`FusedStream`].
///
/// ```
/// # async fn run() -> polyoxide_core::Result<()> {
/// use futures_util::StreamExt as _;
/// use polyoxide_core::pagination::{Paginated, offset_stream};
///
/// // A stand-in for a service's `into_stream()`.
/// let mut stream: Paginated<u64> = offset_stream(0, |offset| async move {
///     Ok(if offset < 3 { vec![offset] } else { Vec::new() })
/// });
/// while let Some(item) = stream.next().await {
///     println!("{}", item?);
/// }
/// # Ok(())
/// # }
/// ```
#[must_use = "streams do nothing unless polled"]
pub struct Paginated<T> {
    // The mutex is never locked by `poll_next`, which has `&mut self` and uses
    // `Mutex::get_mut`. It only makes the type `Sync` without requiring the boxed stream
    // to be `Sync`.
    inner: Mutex<Pin<Box<dyn Stream<Item = Result<T>> + Send + 'static>>>,
    terminated: bool,
}

impl<T> Paginated<T> {
    /// Wraps a stream of items.
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<T>> + Send + 'static,
    {
        Self {
            inner: Mutex::new(Box::pin(stream)),
            terminated: false,
        }
    }
}

impl<T> Stream for Paginated<T> {
    type Item = Result<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.terminated {
            return Poll::Ready(None);
        }
        let inner = this.inner.get_mut().unwrap_or_else(PoisonError::into_inner);
        let item = ready!(inner.as_mut().poll_next(cx));
        if item.is_none() {
            this.terminated = true;
        }
        Poll::Ready(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.terminated {
            return (0, Some(0));
        }
        match self.inner.try_lock() {
            Ok(inner) => inner.size_hint(),
            Err(_) => (0, None),
        }
    }
}

impl<T> FusedStream for Paginated<T> {
    fn is_terminated(&self) -> bool {
        self.terminated
    }
}

impl<T> fmt::Debug for Paginated<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Paginated")
            .field("terminated", &self.terminated)
            .finish_non_exhaustive()
    }
}

/// One page of a cursor-paginated listing, as consumed by [`cursor_stream`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorPage<T> {
    /// The items on this page.
    pub items: Vec<T>,
    /// The cursor for the next page, or `None` if this is the last page.
    pub next_cursor: Option<String>,
}

impl<T> CursorPage<T> {
    /// Creates a page.
    #[must_use]
    pub fn new(items: Vec<T>, next_cursor: Option<String>) -> Self {
        Self { items, next_cursor }
    }
}

/// Lazily walks a cursor-paginated endpoint, yielding every item.
///
/// `fetch` is called with the cursor for the page to fetch (`start` for the first page,
/// `None` meaning "from the beginning") and returns that page. Walking stops when a page
/// has no next cursor, when the next cursor is empty, or when the server repeats the cursor
/// it was just given (which would otherwise loop forever). Service-specific end sentinels
/// (such as the CLOB's `"LTE="`) must be mapped to `None` by `fetch`.
pub fn cursor_stream<T, F, Fut>(start: Option<String>, fetch: F) -> Paginated<T>
where
    T: Send + 'static,
    F: FnMut(Option<String>) -> Fut + Send + 'static,
    Fut: Future<Output = Result<CursorPage<T>>> + Send + 'static,
{
    struct State<T, F> {
        fetch: F,
        cursor: Option<String>,
        buffer: VecDeque<T>,
        done: bool,
    }

    let state = State {
        fetch,
        cursor: start,
        buffer: VecDeque::new(),
        done: false,
    };
    Paginated::new(futures_util::stream::unfold(
        state,
        |mut state| async move {
            loop {
                if let Some(item) = state.buffer.pop_front() {
                    return Some((Ok(item), state));
                }
                if state.done {
                    return None;
                }
                let requested = state.cursor.clone();
                match (state.fetch)(requested.clone()).await {
                    Ok(page) => {
                        state.done = match &page.next_cursor {
                            None => true,
                            Some(next) => next.is_empty() || Some(next) == requested.as_ref(),
                        };
                        state.cursor = page.next_cursor;
                        state.buffer = page.items.into();
                    }
                    Err(err) => {
                        state.done = true;
                        return Some((Err(err), state));
                    }
                }
            }
        },
    ))
}

/// Lazily walks an offset-paginated endpoint, yielding every item.
///
/// `fetch` is called with the offset of the page to fetch, starting at `start`, and
/// returns that page's items. The next offset is the previous one plus the number of items
/// returned. Walking stops at the first empty page (or after the first error).
///
/// A page shorter than the requested `limit` is **not** treated as the last page: a server
/// may cap the page size below the requested `limit`, and stopping there would silently
/// truncate the listing. The cost is one final request that returns an empty page. When an
/// endpoint reports the end explicitly (e.g. `hasMore: false`), `fetch` can return an empty
/// page without sending a request once it has seen that signal.
pub fn offset_stream<T, F, Fut>(start: u64, fetch: F) -> Paginated<T>
where
    T: Send + 'static,
    F: FnMut(u64) -> Fut + Send + 'static,
    Fut: Future<Output = Result<Vec<T>>> + Send + 'static,
{
    struct State<T, F> {
        fetch: F,
        offset: u64,
        buffer: VecDeque<T>,
        done: bool,
    }

    let state = State {
        fetch,
        offset: start,
        buffer: VecDeque::new(),
        done: false,
    };
    Paginated::new(futures_util::stream::unfold(
        state,
        |mut state| async move {
            loop {
                if let Some(item) = state.buffer.pop_front() {
                    return Some((Ok(item), state));
                }
                if state.done {
                    return None;
                }
                match (state.fetch)(state.offset).await {
                    Ok(items) => {
                        let len = u64::try_from(items.len()).unwrap_or(u64::MAX);
                        state.done = len == 0;
                        state.offset = state.offset.saturating_add(len);
                        state.buffer = items.into();
                    }
                    Err(err) => {
                        state.done = true;
                        return Some((Err(err), state));
                    }
                }
            }
        },
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use futures_util::{StreamExt as _, TryStreamExt as _};

    use super::*;
    use crate::{Error, ValidationError};

    #[tokio::test]
    async fn walks_cursor_pages() {
        let stream = cursor_stream(None, |cursor| async move {
            Ok(match cursor.as_deref() {
                None => CursorPage::new(vec![1, 2], Some("a".to_owned())),
                Some("a") => CursorPage::new(vec![], Some("b".to_owned())),
                Some("b") => CursorPage::new(vec![3], None),
                Some(other) => panic!("unexpected cursor {other}"),
            })
        });
        let items: Vec<i32> = stream.try_collect().await.unwrap();
        assert_eq!(items, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn stops_on_repeated_or_empty_cursor() {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let stream = cursor_stream(Some("x".to_owned()), move |_| {
            counter.fetch_add(1, Ordering::SeqCst);
            async { Ok(CursorPage::new(vec![1], Some("x".to_owned()))) }
        });
        let items: Vec<i32> = stream.try_collect().await.unwrap();
        assert_eq!(items, vec![1]);
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let stream = cursor_stream(None, |_| async {
            Ok(CursorPage::new(vec![1], Some(String::new())))
        });
        let items: Vec<i32> = stream.try_collect().await.unwrap();
        assert_eq!(items, vec![1]);
    }

    #[tokio::test]
    async fn yields_error_then_ends() {
        let stream = cursor_stream(None, |_| async {
            Err::<CursorPage<i32>, _>(Error::from(ValidationError::new("x", "boom")))
        });
        let results: Vec<_> = stream.collect().await;
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
    }

    #[tokio::test]
    async fn paginated_is_unpin_fused_and_debug() {
        fn assert_traits<T: Unpin + Send + Sync + 'static + fmt::Debug + FusedStream>() {}
        assert_traits::<Paginated<String>>();

        let mut stream = cursor_stream(None, |cursor| async move {
            Ok(match cursor.as_deref() {
                None => CursorPage::new(vec![1, 2], Some("a".to_owned())),
                _ => CursorPage::new(vec![3], None),
            })
        });
        assert_eq!(format!("{stream:?}"), "Paginated { terminated: false, .. }");
        // The usual loop works without pinning.
        let mut items = Vec::new();
        while let Some(item) = stream.next().await {
            items.push(item.unwrap());
        }
        assert_eq!(items, vec![1, 2, 3]);
        assert!(stream.is_terminated());
        assert!(stream.next().await.is_none());
        assert_eq!(stream.size_hint(), (0, Some(0)));
    }

    #[tokio::test]
    async fn walks_offsets_until_an_empty_page() {
        // Short pages (the server capping `limit`) do not end the walk; only an empty page
        // does.
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let stream = offset_stream(0, move |offset| {
            counter.fetch_add(1, Ordering::SeqCst);
            async move {
                Ok(match offset {
                    0 => vec![1, 2],
                    2 => vec![3],
                    3 => vec![],
                    other => panic!("unexpected offset {other}"),
                })
            }
        });
        let items: Vec<i32> = stream.try_collect().await.unwrap();
        assert_eq!(items, vec![1, 2, 3]);
        assert_eq!(calls.load(Ordering::SeqCst), 3);

        let stream = offset_stream(5, |offset| async move {
            Ok(if offset < 7 { vec![offset] } else { vec![] })
        });
        let items: Vec<u64> = stream.try_collect().await.unwrap();
        assert_eq!(items, vec![5, 6]);
    }

    #[tokio::test]
    async fn offset_stream_yields_error_then_ends() {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let stream = offset_stream(0, move |offset| {
            counter.fetch_add(1, Ordering::SeqCst);
            async move {
                if offset == 0 {
                    Ok(vec![1])
                } else {
                    Err(Error::from(ValidationError::new("x", "boom")))
                }
            }
        });
        let results: Vec<Result<i32>> = stream.collect().await;
        assert_eq!(results.len(), 2);
        assert!(matches!(results.first(), Some(Ok(1))));
        assert!(matches!(results.get(1), Some(Err(Error::Validation(_)))));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}
