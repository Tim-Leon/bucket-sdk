

// Call data loader when loading in a file to be uploaded.
pub trait DataLoader {
    fn load_from_path(&self, );
}