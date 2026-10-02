//! An ordered query-string builder.

use std::fmt::Display;

/// An ordered list of query-string parameters.
///
/// Values are formatted with [`Display`], so enums used as parameters should implement
/// `Display` with their wire spelling. Booleans format as `true`/`false`.
///
/// - [`Query::push_all`] repeats the key for each value (`id=1&id=2`), the OpenAPI default
///   (`style: form, explode: true`) for array parameters.
/// - [`Query::push_csv`] joins values with commas (`token_ids=1,2`), for parameters
///   documented as comma-separated lists.
///
/// ```
/// use marcasite_core::Query;
///
/// let mut q = Query::new();
/// q.push("limit", 10)
///     .push_opt("closed", Some(false))
///     .push_opt::<u32>("offset", None)
///     .push_all("id", [1, 2]);
/// assert_eq!(q.to_string(), "limit=10&closed=false&id=1&id=2");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    pairs: Vec<(&'static str, String)>,
}

impl Query {
    /// Creates an empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends `key=value`.
    pub fn push(&mut self, key: &'static str, value: impl Display) -> &mut Self {
        self.pairs.push((key, value.to_string()));
        self
    }

    /// Appends `key=value` if `value` is `Some`.
    pub fn push_opt<V: Display>(&mut self, key: &'static str, value: Option<V>) -> &mut Self {
        if let Some(value) = value {
            self.push(key, value);
        }
        self
    }

    /// Appends `key=value` once per value (repeated keys).
    pub fn push_all<I>(&mut self, key: &'static str, values: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Display,
    {
        for value in values {
            self.push(key, value);
        }
        self
    }

    /// Appends `key=v1,v2,...` if there is at least one value.
    pub fn push_csv<I>(&mut self, key: &'static str, values: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Display,
    {
        let joined = values
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");
        if !joined.is_empty() {
            self.pairs.push((key, joined));
        }
        self
    }

    /// Replaces every existing value of `key` with `value`.
    pub fn set(&mut self, key: &'static str, value: impl Display) -> &mut Self {
        self.remove(key);
        self.push(key, value)
    }

    /// Removes every value of `key`.
    pub fn remove(&mut self, key: &str) -> &mut Self {
        self.pairs.retain(|(k, _)| *k != key);
        self
    }

    /// Returns the first value of `key`, if any.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.as_str())
    }

    /// `true` if there are no parameters.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// The number of parameters (counting repeated keys).
    #[must_use]
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// Iterates over the `(key, value)` pairs in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &str)> + '_ {
        self.pairs.iter().map(|(k, v)| (*k, v.as_str()))
    }
}

impl std::fmt::Display for Query {
    /// Formats the query as an `application/x-www-form-urlencoded` string (without `?`).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        for (k, v) in &self.pairs {
            serializer.append_pair(k, v);
        }
        f.write_str(&serializer.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_and_encoding() {
        let mut q = Query::new();
        q.push_csv("token_ids", ["1", "2"])
            .push_csv::<[&str; 0]>("empty", [])
            .push("q", "a b&c");
        assert_eq!(q.to_string(), "token_ids=1%2C2&q=a+b%26c");
        assert_eq!(q.get("q"), Some("a b&c"));
    }

    #[test]
    fn set_replaces() {
        let mut q = Query::new();
        q.push("cursor", "a").push("limit", 1).set("cursor", "b");
        assert_eq!(q.to_string(), "limit=1&cursor=b");
    }
}
