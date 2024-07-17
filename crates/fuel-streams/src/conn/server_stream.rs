#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ServerStream {
    pub url: String,
}

#[allow(dead_code)]
impl ServerStream {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}
