//! Gamma API client (`https://gamma-api.polymarket.com`).
//!
//! Covers events, markets, tags, series, comments, sports, search and public profiles.
//! Start from [`GammaClient`].
//!
//! See <https://docs.polymarket.com/api-reference/predictions/overview>.

mod client;
mod tags;

pub use client::{GammaClient, GammaClientBuilder};
pub use tags::{GetTag, ListTags, Tag, TagId};
