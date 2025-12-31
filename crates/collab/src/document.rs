use yrs::Doc;

pub struct CollabDocument {
    doc: Doc,
}

impl CollabDocument {
    pub fn new() -> Self {
        Self { doc: Doc::new() }
    }

    pub fn get_doc(&self) -> &Doc {
        &self.doc
    }
}

impl Default for CollabDocument {
    fn default() -> Self {
        Self::new()
    }
}
