use std::ops::Deref;

#[derive(Debug, Default, Clone)]
pub struct HeaderMap {
    pub headers: Vec<(String, String)>,
}

#[allow(unused)]
impl HeaderMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_header_lines(header_str_lines: &mut dyn Iterator<Item = &str>) -> Option<Self> {
        let headers = header_str_lines.fold(HeaderMap::new(), |mut curr, next| {
            let mut key_val = next.splitn(2, ": ");

            let key = key_val.next();
            let val = key_val.next();

            if let (Some(key), Some(val)) = (key, val) {
                curr.insert(key, val);
            }

            curr
        });

        if !headers.is_empty() {
            Some(headers)
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        let existing = self
            .headers
            .iter_mut()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v);

        if let Some(existing) = existing {
            *existing = value.to_owned();
        } else {
            self.headers.push((key.to_owned(), value.to_owned()));
        }
    }

    pub fn remove(&mut self, key: &str) -> bool {
        let key = key.to_lowercase();

        let existing = self
            .headers
            .iter()
            .position(|(k, _)| k.to_lowercase() == key);

        if let Some(existing) = existing {
            self.headers.remove(existing);

            true
        } else {
            false
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        let key = key.to_lowercase();

        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut String> {
        let key = key.to_lowercase();

        self.headers
            .iter_mut()
            .find(|(k, _)| k.to_lowercase() == key)
            .map(|(_, v)| v)
    }
}

impl Deref for HeaderMap {
    type Target = Vec<(String, String)>;

    fn deref(&self) -> &Self::Target {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use crate::header_map::HeaderMap;

    #[test]
    fn test_insert() {
        let mut header_map = HeaderMap::new();

        header_map.insert("Transfer-Encoding", "chunked");

        assert_eq!(header_map.get("Transfer-Encoding"), Some("chunked"));

        header_map.insert("Transfer-Encoding", "gzip");

        assert_eq!(header_map.get("Transfer-Encoding"), Some("gzip"));
    }

    #[test]
    fn test_remove() {
        let mut header_map = HeaderMap::new();

        header_map.insert("Transfer-Encoding", "chunked");

        assert_eq!(header_map.get("Transfer-Encoding"), Some("chunked"));

        assert_eq!(header_map.remove("Transfer-Encoding"), true);

        assert_eq!(header_map.get("Transfer-Encoding"), None);
    }

    #[test]
    fn test_get() {
        let mut header_map = HeaderMap::new();

        header_map.insert("Transfer-Encoding", "chunked");

        assert_eq!(header_map.get("Transfer-Encoding"), Some("chunked"));
    }

    #[test]
    fn test_get_mut() {
        let mut header_map = HeaderMap::new();

        header_map.insert("Transfer-Encoding", "chunked");

        assert_eq!(header_map.get("Transfer-Encoding"), Some("chunked"));

        if let Some(te_mut) = header_map.get_mut("Transfer-Encoding") {
            *te_mut = "compress".to_owned();
        }

        assert_eq!(header_map.get("Transfer-Encoding"), Some("compress"));
    }
}
