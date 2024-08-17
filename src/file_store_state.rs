/// Will contain the local file store state.
/// Will update, according to webhooks and other such functionality.
/// Could be though of as a virtual filesystem representing the cloud storage system, and used to map cloud to local.
pub struct FileSystemState {}

pub struct FileState {
    pub name: String,
    pub path: String,
}

pub trait FileStoreStates {
    fn index(&mut self);
    fn update(&mut self);
    fn delete(&mut self);
}
