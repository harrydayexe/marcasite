//! Client-side request rate limiting.
//!
//! [`RateLimit`] is the public configuration; the limiter that enforces it is internal and
//! owned by the [`HttpClient`](crate::HttpClient).

use std::{
    collections::BTreeSet,
    num::NonZeroU32,
    sync::{Mutex as StdMutex, MutexGuard, PoisonError},
    time::Duration,
};

use tokio::time::Instant;

use crate::error::ValidationError;

/// The longest accepted accounting period for [`RateLimit::new`], and the longest burst
/// allowance that is ever accumulated.
const MAX_PERIOD: Duration = Duration::from_secs(24 * 60 * 60);

/// A client-side request budget: at most `requests` HTTP requests per period, with an
/// optional burst allowance.
///
/// Install it with [`HttpClientBuilder::rate_limit`](crate::HttpClientBuilder::rate_limit).
/// Every HTTP attempt, **including automatic retries**, takes one slot before it is sent;
/// when the budget is exhausted the request waits (asynchronously, on [`tokio::time`]) for
/// its turn instead of failing. Waiting callers are served in the order they arrived, and a
/// caller that stops polling its request (for example a pagination stream left unpolled)
/// does not hold up the others.
///
/// The limiter is a token bucket in its GCRA form: requests are spaced `per / requests`
/// apart, and up to `burst` requests may be sent back to back after an idle period. The
/// default burst is `1`, which spaces requests evenly and never sends two at once; raise it
/// with [`with_burst`](Self::with_burst) to allow short spikes while keeping the same
/// average rate.
///
/// The budget is shared by every clone of the [`HttpClient`](crate::HttpClient) it was
/// configured on, so one limit spans all service clients built from that client. Separate
/// `HttpClient`s (including those a service client builds for itself) have separate
/// budgets. It covers REST requests only; WebSocket connections do not go through it.
///
/// Cancellation is free: a request that is dropped while waiting for its turn (for example
/// by a caller's own timeout) gives its slot to the next new request, so no capacity is
/// lost. Requests already waiting keep their own turns.
///
/// Spacing is `per / requests` rounded down to whole nanoseconds (at least 1 ns), so at odd
/// rates it can be marginally faster than configured, and rates above one request per
/// nanosecond are capped at that rate.
///
/// # Timeouts
///
/// - The wait for the limiter does **not** count against the per-request
///   [`timeout`](crate::HttpClientBuilder::timeout), which starts when the request is sent.
/// - A timeout the caller wraps around a request (e.g. `tokio::time::timeout`) **does**
///   include the wait.
/// - A `429` / `Retry-After` is not fed back into the limiter: a retry first sleeps for the
///   backoff and then waits for a slot, so the two delays add up.
///
/// ```
/// use std::num::NonZeroU32;
/// use marcasite_core::{HttpClient, RateLimit};
///
/// # fn main() -> marcasite_core::Result<()> {
/// // 10 requests per second, bursts of up to 5, shared by every service client built
/// // from `http` (in the `marcasite` crate, pass it to `Polymarket::builder().http_client`).
/// let limit = RateLimit::per_second(NonZeroU32::new(10).unwrap())
///     .with_burst(NonZeroU32::new(5).unwrap());
/// let http = HttpClient::builder().rate_limit(limit).build()?;
/// assert!(http.has_rate_limit());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RateLimit {
    requests: NonZeroU32,
    per: Duration,
    burst: NonZeroU32,
}

impl RateLimit {
    /// At most `requests` requests per second, with a burst of one.
    #[must_use]
    pub const fn per_second(requests: NonZeroU32) -> Self {
        Self {
            requests,
            per: Duration::from_secs(1),
            burst: NonZeroU32::MIN,
        }
    }

    /// At most `requests` requests per `per`, with a burst of one.
    ///
    /// # Errors
    ///
    /// Returns a [`ValidationError`] if `per` is zero or longer than 24 hours.
    pub fn new(requests: NonZeroU32, per: Duration) -> Result<Self, ValidationError> {
        if per.is_zero() {
            return Err(ValidationError::new(
                "per",
                "the rate limit period must be greater than zero",
            ));
        }
        if per > MAX_PERIOD {
            return Err(ValidationError::new(
                "per",
                "the rate limit period must be at most 24 hours",
            ));
        }
        Ok(Self {
            requests,
            per,
            burst: NonZeroU32::MIN,
        })
    }

    /// Allows up to `burst` requests to be sent back to back after an idle period (default
    /// 1). The average rate is unchanged. The effective burst is capped so that
    /// `(burst - 1) * interval` is at most 24 hours.
    #[must_use]
    pub const fn with_burst(mut self, burst: NonZeroU32) -> Self {
        self.burst = burst;
        self
    }

    /// The number of requests allowed per [`per`](Self::per).
    #[must_use]
    pub const fn requests(&self) -> NonZeroU32 {
        self.requests
    }

    /// The period the request count refers to.
    #[must_use]
    pub const fn per(&self) -> Duration {
        self.per
    }

    /// The number of requests that may be sent back to back.
    #[must_use]
    pub const fn burst(&self) -> NonZeroU32 {
        self.burst
    }
}

/// The shared limiter (GCRA): `tat` is the theoretical arrival time of the next request under
/// a perfectly smooth stream.
///
/// A caller reserves its slot under a short, synchronous critical section and then sleeps
/// without holding any lock, so a waiter that stops being polled never blocks the others. A
/// waiter dropped before its slot opens returns the slot as a hole, which the next caller
/// takes instead of extending the schedule; the set of send times therefore never changes,
/// and cancellation stays free without ever exceeding the rate.
#[derive(Debug)]
pub(crate) struct Limiter {
    /// Spacing between requests (at least 1 ns).
    interval: Duration,
    /// How far ahead of `tat` a request may be sent: `(burst - 1) * interval`.
    tolerance: Duration,
    state: StdMutex<State>,
}

#[derive(Debug, Default)]
struct State {
    tat: Option<Instant>,
    /// Open times of reserved slots whose waiter was dropped before they opened.
    holes: BTreeSet<Instant>,
}

impl Limiter {
    pub(crate) fn new(limit: RateLimit) -> Self {
        let interval = (limit.per / limit.requests.get()).max(Duration::from_nanos(1));
        let tolerance = interval
            .saturating_mul(limit.burst.get().saturating_sub(1))
            .min(MAX_PERIOD);
        Self {
            interval,
            tolerance,
            state: StdMutex::new(State::default()),
        }
    }

    fn state(&self) -> MutexGuard<'_, State> {
        // The critical sections cannot panic, but never propagate poisoning regardless.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Reserves the next slot and returns the instant it opens (possibly in the past).
    fn reserve(&self, now: Instant) -> Instant {
        let mut state = self.state();
        // Holes whose time has passed can no longer be used without bunching requests.
        state.holes = state.holes.split_off(&now);
        if let Some(open_at) = state.holes.pop_first() {
            return open_at;
        }
        let base = match state.tat {
            Some(t) if t > now => t,
            _ => now,
        };
        // Saturate on (absurd) overflow: leave the state unchanged.
        state.tat = Some(base.checked_add(self.interval).unwrap_or(base));
        base.checked_sub(self.tolerance).unwrap_or(now)
    }

    /// Waits until a slot is available and takes it.
    ///
    /// Cancel safe: if the future is dropped before its slot opens, the slot goes to the next
    /// caller.
    pub(crate) async fn acquire(&self) {
        let now = Instant::now();
        let open_at = self.reserve(now);
        if open_at <= now {
            return;
        }
        tracing::debug!(
            wait_ms = u64::try_from(open_at.duration_since(now).as_millis()).unwrap_or(u64::MAX),
            "waiting on the client-side rate limit"
        );
        let mut guard = Reservation {
            limiter: self,
            open_at,
            used: false,
        };
        tokio::time::sleep_until(open_at).await;
        guard.used = true;
    }
}

/// Returns an unused slot to the limiter when a waiter is dropped before its slot opens.
struct Reservation<'a> {
    limiter: &'a Limiter,
    open_at: Instant,
    used: bool,
}

impl Drop for Reservation<'_> {
    fn drop(&mut self) {
        if !self.used && self.open_at > Instant::now() {
            self.limiter.state().holes.insert(self.open_at);
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing)]

    use std::sync::Arc;

    use super::*;

    fn nz(n: u32) -> NonZeroU32 {
        NonZeroU32::new(n).unwrap()
    }

    fn limiter(rate: u32, burst: u32) -> Arc<Limiter> {
        Arc::new(Limiter::new(
            RateLimit::per_second(nz(rate)).with_burst(nz(burst)),
        ))
    }

    #[test]
    fn validation() {
        assert!(RateLimit::new(nz(5), Duration::ZERO).is_err());
        assert!(RateLimit::new(nz(5), MAX_PERIOD + Duration::from_secs(1)).is_err());
        let ok = RateLimit::new(nz(5), Duration::from_secs(60)).unwrap();
        assert_eq!(ok.burst().get(), 1);
        assert_eq!(ok.requests().get(), 5);
        // Extreme rates and bursts do not panic.
        let l = Limiter::new(RateLimit::per_second(nz(u32::MAX)).with_burst(nz(u32::MAX)));
        assert!(l.interval >= Duration::from_nanos(1));
        assert!(l.tolerance <= MAX_PERIOD);
    }

    #[tokio::test(start_paused = true)]
    async fn smooth_spacing_and_burst() {
        let l = limiter(10, 3);
        let start = Instant::now();
        for _ in 0..3 {
            l.acquire().await;
        }
        assert_eq!(start.elapsed(), Duration::ZERO, "burst is immediate");
        l.acquire().await;
        assert!(start.elapsed() >= Duration::from_millis(100));
        assert!(start.elapsed() < Duration::from_millis(110));
    }

    #[tokio::test(start_paused = true)]
    async fn hundred_acquisitions_at_ten_per_second() {
        let l = limiter(10, 1);
        let start = Instant::now();
        for _ in 0..100 {
            l.acquire().await;
        }
        let took = start.elapsed();
        assert!(took >= Duration::from_millis(9_900), "{took:?}");
        assert!(took < Duration::from_millis(10_100), "{took:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn idle_limiter_does_not_wait() {
        let l = limiter(10, 1);
        l.acquire().await;
        tokio::time::sleep(Duration::from_secs(5)).await;
        let start = Instant::now();
        l.acquire().await;
        assert_eq!(start.elapsed(), Duration::ZERO);
    }

    #[tokio::test(start_paused = true)]
    async fn burst_refills_but_never_exceeds_burst() {
        let l = limiter(10, 3);
        for _ in 0..3 {
            l.acquire().await;
        }
        // Idle far longer than needed to refill: still only 3 immediate slots.
        tokio::time::sleep(Duration::from_secs(60)).await;
        let start = Instant::now();
        for _ in 0..3 {
            l.acquire().await;
        }
        assert_eq!(start.elapsed(), Duration::ZERO);
        l.acquire().await;
        assert!(start.elapsed() >= Duration::from_millis(100));
        // Partial refill: after 200 ms two slots are back.
        tokio::time::sleep(Duration::from_millis(200)).await;
        let start = Instant::now();
        l.acquire().await;
        l.acquire().await;
        assert_eq!(start.elapsed(), Duration::ZERO);
    }

    async fn run_concurrently(l: &Arc<Limiter>, tasks: usize) -> Vec<Instant> {
        let mut handles = Vec::new();
        for _ in 0..tasks {
            let l = Arc::clone(l);
            handles.push(tokio::spawn(async move {
                l.acquire().await;
                Instant::now()
            }));
        }
        let mut times = Vec::new();
        for h in handles {
            times.push(h.await.unwrap());
        }
        times.sort();
        times
    }

    #[tokio::test(start_paused = true)]
    async fn concurrent_tasks_never_exceed_the_rate() {
        for burst in [1u32, 4] {
            let l = limiter(10, burst);
            let times = run_concurrently(&l, 40).await;
            let t = Duration::from_millis(100);
            for i in 0..times.len() {
                for j in i..times.len() {
                    let window = times[j] - times[i];
                    let count = u32::try_from(j - i + 1).unwrap();
                    // GCRA bound: at most `burst` plus one per interval in any window.
                    let allowed = burst + u32::try_from(window.as_nanos() / t.as_nanos()).unwrap();
                    assert!(count <= allowed, "burst {burst}: {count} in {window:?}");
                }
            }
            let total = times[39] - times[0];
            assert!(
                total >= Duration::from_millis(100) * (40 - burst),
                "{total:?}"
            );
        }
    }

    #[tokio::test(start_paused = true)]
    async fn cancelled_waiters_take_no_slot() {
        let l = limiter(10, 1);
        l.acquire().await;
        let mut handles = Vec::new();
        for _ in 0..50 {
            let l = Arc::clone(&l);
            handles.push(tokio::spawn(async move {
                tokio::time::timeout(Duration::from_millis(10), l.acquire())
                    .await
                    .is_err()
            }));
        }
        for h in handles {
            assert!(h.await.unwrap(), "waiter should have timed out");
        }
        let start = Instant::now();
        l.acquire().await;
        assert!(
            start.elapsed() <= Duration::from_millis(100),
            "{:?}",
            start.elapsed()
        );
    }

    #[tokio::test(start_paused = true)]
    async fn waiters_are_served_in_fifo_order() {
        let l = limiter(10, 1);
        l.acquire().await;
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut handles = Vec::new();
        for i in 0..10u32 {
            let (l, order) = (Arc::clone(&l), Arc::clone(&order));
            handles.push(tokio::spawn(async move {
                l.acquire().await;
                order.lock().unwrap().push(i);
            }));
            // Let the task reach the lock before spawning the next.
            tokio::task::yield_now().await;
        }
        for h in handles {
            h.await.unwrap();
        }
        assert_eq!(*order.lock().unwrap(), (0..10).collect::<Vec<_>>());
    }

    #[tokio::test(start_paused = true)]
    async fn unpolled_waiter_does_not_block_others() {
        let l = limiter(10, 1);
        l.acquire().await;
        // Start a waiter, poll it once, then keep it alive without polling it.
        let mut held = Box::pin(l.acquire());
        assert!(
            tokio::time::timeout(Duration::from_millis(1), &mut held)
                .await
                .is_err()
        );
        let start = Instant::now();
        tokio::time::timeout(Duration::from_secs(1), l.acquire())
            .await
            .expect("a second waiter must not be blocked by an unpolled one");
        assert!(start.elapsed() >= Duration::from_millis(150));
        drop(held);
    }

    #[tokio::test(start_paused = true)]
    async fn holes_are_reused_without_exceeding_the_rate() {
        let l = limiter(10, 1);
        l.acquire().await;
        // Reserve slots at 100 ms and 200 ms; cancel the first.
        let mut first = Box::pin(l.acquire());
        assert!(
            tokio::time::timeout(Duration::from_millis(1), &mut first)
                .await
                .is_err()
        );
        let second = tokio::spawn({
            let l = Arc::clone(&l);
            async move {
                l.acquire().await;
                Instant::now()
            }
        });
        tokio::task::yield_now().await;
        drop(first);
        let start = Instant::now();
        l.acquire().await; // takes the freed 100 ms slot
        let third = Instant::now();
        let second = second.await.unwrap();
        assert!(third - start < Duration::from_millis(100));
        assert!(second - third >= Duration::from_millis(100));
        // The schedule continues after the last reservation.
        l.acquire().await;
        assert!(Instant::now() - second >= Duration::from_millis(100));
    }
}
