/// Adapter utilities that translate request DTOs into HTTP query parameters.
///
/// The builder keeps the transformation logic in one place so consumers
/// avoid duplicating `Option` trimming and string formatting.
#[derive(Default)]
pub struct QueryParamsBuilder {
    params: Vec<(&'static str, String)>,
}

impl QueryParamsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a value if present, formatting it using `ToString`.
    pub fn push_value<T: ToString>(&mut self, key: &'static str, value: Option<T>) -> &mut Self {
        if let Some(value) = value {
            self.params.push((key, value.to_string()));
        }
        self
    }

    /// Push a trimmed string if present and not empty.
    pub fn push_trimmed(&mut self, key: &'static str, value: &Option<String>) -> &mut Self {
        if let Some(clean) = value
            .as_ref()
            .map(String::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            self.params.push((key, clean.to_string()));
        }
        self
    }

    pub fn into_vec(self) -> Vec<(&'static str, String)> {
        self.params
    }
}

/// Adapter contract for translating link list inputs into HTTP query params.
pub trait LinkListQueryAdapter {
    fn to_query_params(&self) -> Vec<(&'static str, String)>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_skips_blank_strings() {
        let mut builder = QueryParamsBuilder::new();
        builder
            .push_trimmed("slug", &Some("  ".to_string()))
            .push_trimmed("target_contains", &Some("abc".to_string()));

        let params = builder.into_vec();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], ("target_contains", "abc".to_string()));
    }

    #[test]
    fn builder_handles_numbers_and_bool() {
        let mut builder = QueryParamsBuilder::new();
        builder
            .push_value("page", Some(2u32))
            .push_value("per_page", Some(10))
            .push_value("active", Some(false));

        let params = builder.into_vec();
        assert_eq!(params.len(), 3);
        assert_eq!(params[0], ("page", "2".to_string()));
        assert_eq!(params[1], ("per_page", "10".to_string()));
        assert_eq!(params[2], ("active", "false".to_string()));
    }
}
