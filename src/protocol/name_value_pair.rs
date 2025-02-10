pub struct NameValuePair {
    pub name: String,
    pub value: String,
}

impl NameValuePair {
    pub fn new(name: &str, value: &str) -> Self {
        return NameValuePair {
            name: String::from(name),
            value: String::from(value),
        };
    }
}
