//! Turn paginated endpoints into [`Stream`]s of items.
//!
//! Service clients expose each paginated endpoint twice:
//!
//! - `send()` fetches a single page (raw page access, including the cursor), and
//! - `into_stream()` walks every page lazily and yields individual items.
//!
//! The helpers here implement the walking for the two pagination styles the APIs use:
//! opaque cursors ([`cursor_stream`]) and numeric offsets ([`offset_stream`]).
//!
//! Streams stop after the first error (which is yielded). Use
//! [`futures_util::StreamExt::take`] to bound the number of items, and
//! [`futures_util::TryStreamExt::try_collect`] to gather them.
//!
//! [`Stream`]: futures_core::Stream

use std::{collections::VecDeque, future::Future};

use futures_core::Stream;

use crate::Result;

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
pub fn cursor_stream<T, F, Fut>(
    start: Option<String>,
    fetch: F,
) -> impl Stream<Item = Result<T>> + Send + 'static
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
    futures_util::stream::unfold(state, |mut state| async move {
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
    })
}

/// Lazily walks an offset-paginated endpoint, yielding every item.
///
/// `fetch` is called with the offset of the page to fetch, starting at `start`, and
/// returns that page's items. The next offset is the previous one plus the number of items
/// returned. Walking stops at the first empty page, or at the first page shorter than
/// `page_size` when the page size is known.
pub fn offset_stream<T, F, Fut>(
    start: u64,
    page_size: Option<u64>,
    fetch: F,
) -> impl Stream<Item = Result<T>> + Send + 'static
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
    futures_util::stream::unfold(state, move |mut state| async move {
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
                    state.done = len == 0 || page_size.is_some_and(|size| len < size);
                    state.offset = state.offset.saturating_add(len);
                    state.buffer = items.into();
                }
                Err(err) => {
                    state.done = true;
                    return Some((Err(err), state));
                }
            }
        }
    })
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
    async fn walks_offsets() {
        let stream = offset_stream(0, Some(2), |offset| async move {
            Ok(match offset {
                0 => vec![1, 2],
                2 => vec![3],
                other => panic!("unexpected offset {other}"),
            })
        });
        let items: Vec<i32> = stream.try_collect().await.unwrap();
        assert_eq!(items, vec![1, 2, 3]);

        let stream = offset_stream(5, None, |offset| async move {
            Ok(if offset < 7 { vec![offset] } else { vec![] })
        });
        let items: Vec<u64> = stream.try_collect().await.unwrap();
        assert_eq!(items, vec![5, 6]);
    }
}
