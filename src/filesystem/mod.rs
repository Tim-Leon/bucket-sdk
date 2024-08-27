// We should index out the local filesystem.
// All the files should be added to a btree in order for synchronization
// to work between the filesystem and cloud.

use crate::io::FileWrapper;

pub trait LocalFilesystem {

}

pub trait VirtualFile {
}

pub trait Filesystem<F: FileWrapper> {
    fn get_file(&self, path: String);
    fn delete_file(&self, file: F);
    fn update(&self, file:F);
}