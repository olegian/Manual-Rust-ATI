#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Tag {
    addr: String,
}

impl Tag {
    pub fn new<T>(value: &T) -> Self {
        Tag {
            addr: format!("{:p}", value),
        }
    }
}
