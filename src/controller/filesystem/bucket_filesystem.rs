// use std::collections::BTreeMap;
// use std::error::Error;
// use std::str::FromStr;
//
// use argon2::{Argon2, PasswordHasher};
// use argon2::password_hash::SaltString;
// use bucket_common_types::BucketEncryption;
// use infer::Type;
// use mime::Mime;
// use web_sys::{HtmlInputElement, ReadableStream};
//
// use crate::encryption_v1::PasswordHashErrors;
// use crate::query_client::{
//     backend_api::{BucketDetails, DownloadBucketRequest, GetBucketFilestructureRequest},
//     QueryClient,
// };
//
// use super::bucket::{BucketFileDownloadHandlerDyn, BucketFileUploadHandlerDyn, bucket_download, upload, UploadError};
//
// // Different type for WASM and NATIVE.
// // Acts as a handler for files that are in the cloud.
// // Have a file handler that optionally refer to a local file corresponding to the cloud.
// pub struct VirtualBucketFile<FileHandle> {
//     pub name: String,
//     pub date: time::OffsetDateTime,
//     pub size_in_bytes: u64,
//     pub file_handle: Option<FileHandle>,
// }
//
// #[derive(thiserror::Error, Debug)]
// pub enum WebBucketFileError {
//     #[error("No file handle")]
//     NoFileHandle,
//     #[error("Unknown file type")]
//     UnknownFileType,
//     #[error("Empty")]
//     Empty,
//     #[error("No extension")]
//     NoExtension,
// }
//
// type WebBucketFile = VirtualBucketFile<web_sys::HtmlInputElement>;
//
// trait BucketFileTrait {
//     type Error;
//     type FileHandle;
//     fn is_readable(&self) -> bool;
//     fn files(&self) -> Option<Self::FileHandle>;
//     fn read_chunk(&self, size: i32, offset: u32) -> Result<Vec<u8>, Self::Error>;
//     fn read_stream(&self) -> Result<ReadableStream, Self::Error>;
//     fn get_extension(&self) -> Result<String, Self::Error>;
//     /// Get the mime-type from the extension.
//     fn get_mime_type(&self) -> Result<Mime, Self::Error>;
//     /// Uses the first couple of bytes in the file ot determine the mime-type
//     fn infer_mime_type(&self) -> Result<infer::Type, Self::Error>;
//     fn write_chunk(&self);
//     fn write_stream(&self);
// }
//
// impl BucketFileTrait for WebBucketFile {
//     type Error = WebBucketFileError;
//     // Remember read is for uploading and write is for downloading. Kinda reversed if you think about it.
//     fn is_readable(&self) -> bool {
//         match self.file_handle {
//             None => { return false; }
//             Some(_) => { return true; }
//         };
//     }
//     fn read_chunk(&self, size: u32, offset:u32) -> Result<Vec<u8>, Self::Error> {
//         return match &self.file_handle {
//             Some(x) => {
//                 let file = x.files().unwrap();
//                 let rs = file.get(0).unwrap().stream();
//                 let start = size - offset;
//                 let str = file.get(offset).unwrap().slice_with_i32(i32::try_from(start).unwrap()).unwrap().array_buffer().as_string();
//                 match str {
//                     None => {
//                         Err(WebBucketFileError::Empty)
//                     }
//                     Some(str) => {
//                         Ok(str.into_bytes())
//                     }
//                 }
//             }
//             None => {
//                 // Can not read from file that does not have a corresponding handle attached.
//                 Err(WebBucketFileError::NoFileHandle)
//             }
//         };
//     }
//
//     fn read_stream(&self) -> Result<ReadableStream, Self::Error> {
//         return match &self.file_handle {
//             Some(x) => {
//                 let file = x.files().unwrap();
//                 let rs = file.get(0).unwrap().stream();
//                 Ok(rs)
//             }
//             None => {
//                 // Can not read from file that does not have a corresponding handle attached.
//                 Err(WebBucketFileError::NoFileHandle)
//             }
//         };
//     }
//
//     fn write_chunk(&self) {
//
//     }
//
//
//      fn get_extension(&self) -> Result<String, Self::Error> {
//         let extension = self.name.rsplit_once('.').ok_or(WebBucketFileError::NoExtension)?;
//         let (extension, _) = extension; // Unwrap the result
//         Ok(extension.to_string())
//     }
//
//     // Checks file extension to get mime type.
//      fn get_mime_type(&self) -> Result<Mime, Self::Error> {
//         let extension = self.get_extension().unwrap();
//         let mime = mime::Mime::from_str(extension.as_str()).unwrap();
//         Ok(mime)
//     }
//     //Checks the first couple of bytes of the file to get mime type.
//     fn infer_mime_type(&self) -> Result<infer::Type,Self::Error>{
//         return match &self.file_handle {
//             None => { Err(WebBucketFileError::NoFileHandle) }
//             Some(handle) => {
//                 let buf = self.read_chunk(16,0).unwrap();
//                 let kind = infer::get(&buf);
//                 return match kind {
//                     None => { Err(WebBucketFileError::UnknownFileType) }
//                     Some(kind) => {
//                         Ok(kind)
//                     }
//                 };
//             }
//         }
//     }
// }
//
// type NativeBucketFile = VirtualBucketFile<std::fs::File>;
//
// pub enum VirtualBucketFiles {
//     WebBucketFile(WebBucketFile),
//     NativeBucketFile(NativeBucketFile),
// }
//
//
// // When downloading a file it will be 'xxx.tmp' and then converted to the correct type after completing.
// // Also add support for compression, this will be done by adding a .zip extension to the file.
//
// /// Stores an in sync copy of the buckets filesystem.
// /// This is used to keep track of the files in the bucket.
// /// And is the higher level abstraction of the filesystem.
// /// All actions are performed against the
// struct BucketVirtualFilesystemManagerWeb {
//     //type on_event_callback_fn = fn(BucketEvent);
//     pub files: BTreeMap<String, VirtualBucketFiles>,
//     pub client: QueryClient,
//     pub bucket_id: uuid::Uuid,
//     pub bucket_owner_id: uuid::Uuid,
//     pub encryption: Option<BucketEncryption>,
//     pub hashed_password: Option<String>,
//
//     //pub on_event: Option<Vec<on_event_callback_fn>>,
// }
//
// #[derive(thiserror::Error, Debug)]
// pub enum BucketVirtualFilesystemManagerError {
//     #[error("")]
//     FileNotFound,
//     #[error("")]
//     FileAlreadyExists,
//     #[error("")]
//     DirectoryNotFound,
//     #[error("")]
//     DirectoryAlreadyExists,
//     #[error(transparent)]
//     UploadError(#[from] UploadError),
//     #[error("Failed hash password")]
//     FailedHashPassword,
// }
//
// #[derive(thiserror::Error, Debug)]
// pub enum BucketPasswordHashError {}
//
// pub fn bucket_hash_password(bucket_id_salt: &uuid::Uuid, password: &str) -> Option<String> {
//     //TODO:FIX
//
//     let salt = SaltString::from_b64(&str::from_utf8(bucket_id_salt.as_bytes())?) //TODO: Replace with SALT why use utf8???  TODO: Same in encryption_v1.rs
//         .map_err(|err| PasswordHashErrors::PasswordHashError(err))?;
//     let argon2id = Argon2::default();
//     let password_hash = argon2id
//         .hash_password(password.as_bytes(), salt.as_salt())
//         .map_err(|err| PasswordHashErrors::PasswordHashError(err))?;
// }
//
//
// //TODO: implement observer pattern for filesystem events.
// impl BucketVirtualFilesystemManagerWeb {
//     //type Error = WebBucketFileError;
//     //type FileHandle = WebBucketFile;
//     fn new(
//         client: QueryClient,
//         target_bucket_id: uuid::Uuid,
//         target_bucket_owner_id: uuid::Uuid,
//         password: Option<String>,
//     ) -> Self {
//         Self {
//             files: BTreeMap::new(),
//             client,
//             bucket_id: target_bucket_id,
//             bucket_owner_id: target_bucket_owner_id,
//             hashed_password: match password {
//                 None => {
//                     None
//                 }
//                 Some(password) => {
//                     Some(bucket_hash_password(&target_bucket_id, password.as_str()).ok_or(BucketVirtualFilesystemManagerError::FailedHashPassword).unwrap())
//                 }
//             },
//             encryption: None,
//         }
//     }
//     //Theses methods are private, and are only meant to change the virtual filesystem.
//     async fn virtual_filesystem_remove_file(&mut self, filename: &str) -> Option<VirtualBucketFiles> {
//         let val = self.files.remove(&filename.to_string());
//         return val;
//     }
//
//     async fn virtual_filesystem_add_file(&self, filename: &str) {
//         let val = VirtualBucketFile {
//             web_sys: todo!(),
//             FileSystemHandle: todo!(),
//             name: todo!(),
//             date: todo!(),
//             size_in_bytes: todo!(),
//             file_handle: todo!(),
//         };
//         let key = filename.to_string();
//         self.files.insert(key, val);
//     }
//
//
//     async fn add_files(&mut self, target_directory: String, files: Vec<VirtualBucketFiles>) -> Result<(), WebBucketFileError> {
//         let total_size_in_bytes = files.iter().map(|f| f.size_in_bytes).sum() as usize;
//         let upload_handle = BucketFileUploadHandlerDyn::new();
//
//         upload(self.client,
//                &self.bucket_owner_id,
//                &self.bucket_id, target_directory,
//                files,
//                self.encryption,
//                total_size_in_bytes,
//                upload_handle).await?;
//         for file in files {
//             self.files.insert(file.name, file);
//         }
//         Ok(())
//     }
//
//     async fn remove_files(&mut self, directory: Option<String>, file_names: Vec<String>) -> Result<(), BucketVirtualFilesystemManagerError> {
//         let resulting_filenames: Vec<String> = match directory {
//             Some(x) => {
//                 return file_names.iter().map(|file| x + file.as_str()).collect();
//             }
//             None => file_names,
//         };
//         for file in resulting_filenames {
//             self.files.remove(file.as_str());
//         }
//     }
//
//     async fn download_files(&mut self, files: Vec<VirtualBucketFiles>, details: BucketDetails, downloader: BucketFileDownloadHandlerDyn) -> Result<(), WebBucketFileError> {
//         for file in files {
//             bucket_download(
//                 &mut self.client,
//                 &self.bucket_owner_id,
//                 &self.bucket_id,
//                 self.hashed_password,
//                 details.encryption,
//                 true,
//                 downloader,
//             ).await?;
//         }
//     }
//     // Download the entire bucket.
//     pub async fn download_bucket(&mut self) {
//         self.client.download_bucket(DownloadBucketRequest {
//             bucket_id: self.bucket_id.to_string(),
//             bucket_owner_id: self.bucket_owner_id.to_string(),
//             hashed_password: self.hashed_password.clone(),
//             format: None,
//         })
//     }
//
//     pub async fn upload_files(&mut self, source_files: Vec<VirtualBucketFiles>, target_directory: String) -> Result<(), WebBucketFileError> {
//         let upload_handler = BucketFileUploadHandlerDyn::new();
//         let total_size_in_bytes = source_files.iter().map(|f| f.size_in_bytes).sum() as usize;
//         let file_names = source_files.iter().map(|file| file.name).collect();
//         for file in source_files {}
//         upload(
//             &mut self.client,
//             &self.bucket_owner_id,
//             &self.bucket_id,
//             target_directory,
//             file_names,
//             self.encryption,
//             total_size_in_bytes,
//             upload_handler,
//         );
//     }
//
//     pub async fn update(
//         &mut self,
//         bucket_owner_id: Option<uuid::Uuid>,
//         bucket_id: uuid::Uuid,
//         start_directory: Option<String>,
//     ) -> Result<(), WebBucketFileError> {
//         let file_structure = QueryClient::get_bucket_filestructure(
//             &mut self.client,
//             GetBucketFilestructureRequest {
//                 bucket_id: bucket_id.to_string(),
//                 bucket_owner_id: match bucket_owner_id {
//                     Some(x) => Some(x.to_string()),
//                     None => None,
//                 },
//                 continuation_token: None,
//                 start_directory,
//             },
//         )
//             .await
//             .unwrap();
//         let filesystem = file_structure.into_inner().filesystem;
//     }
//
//     pub async fn sync(&mut self) -> Result<(), WebBucketFileError> {
//         // TODO: Subscribe to event bucket events.
//         todo!()
//     }
//
//
//     pub async fn event_handler(&mut self) -> Result<(), WebBucketFileError> {
//         // TODO: Give user the ability to make there own event handler?.
//         todo!()
//     }
//
//
//     // pub fn get_directory() -> BucketDirectory {
//     //     BucketDirectory { files: Vec::new() }
//     // }
//     // Takes an event.
//     pub fn on_update(&mut self) {
//         todo!()
//     }
// }
