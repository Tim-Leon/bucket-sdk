use byte_unit::TryFromIntError;
use futures::SinkExt;
use futures::TryStreamExt;
use gloo::file::BlobContents;
use mime::{FromStrError, Mime};
use wasm_bindgen::{JsCast, JsValue};
use wasm_streams::{ReadableStream, WritableStream};
use web_sys::js_sys::Uint8Array;

use std::io::{Read, Write};
use std::vec;
use web_sys::HtmlInputElement;

use super::file::BucketFileTrait;

#[derive(thiserror::Error, Debug)]
pub enum WebBucketFileError {
    #[error("No file handle")]
    NoFileHandle,
    #[error("Unknown file type")]
    UnknownFileType,
    #[error("Empty")]
    Empty,
    #[error("No extension")]
    NoExtension,
    #[error(transparent)]
    TryFromIntError(#[from] TryFromIntError),
    #[error(transparent)]
    FromStrError(#[from] FromStrError),

}


#[derive(thiserror::Error, Debug)]
pub enum ConvertHtmlInputElementToFileListError {
    // Is this even possible?
    #[error("No files selected")]
    Empty,
}

fn convert_html_input_element_to_file_list(
    input: web_sys::HtmlInputElement,
) -> Result<gloo::file::FileList, ConvertHtmlInputElementToFileListError> {
    //let el: web_sys::HtmlInputElement = input.target_unchecked_into();
    match input.files() {
        Some(files) => {
            if files.length() < 1 {
                return Err(ConvertHtmlInputElementToFileListError::Empty);
            }
            return Ok(files.into());
        }
        None => return Err(ConvertHtmlInputElementToFileListError::Empty),
    }
}

pub type WebFileHandle = gloo::file::File;

pub struct VirtualWebBucketFile {
    //pub file_handle: Option<web_sys::HtmlInputElement>,
    //pub virtual_file_details: Arc<VirtualFileDetails>,
    file_handle: WebFileHandle,
    filename: String,
    file_type: Mime,
}

#[async_trait(?Send)]
impl BucketFileTrait for VirtualWebBucketFile {
    type Error = WebBucketFileError;

    type FileHandle = WebFileHandle;
    fn new(filename: &str, mime:&Mime) -> Result<Self, Self::Error> where Self: Sized {
        let file_handle = gloo::file::File::new_with_options(
            &filename,
            "",
            Some(mime.to_string().as_str()),
            None,
        );
        Ok(Self {
            file_handle,
            filename: filename.to_string(),
            file_type: mime,
        })
    }
    fn from(file_handle: Self::FileHandle, filename: String, mime: &Mime) -> Self {
        Self {
            file_handle,
            filename,
            file_type: mime,
        }
    }

    fn get_file_handle(&self) -> Self::FileHandle {
        self.file_handle
    }
    async fn read_chunk(&self, size: u64, offset: u64) -> Result<Vec<u8>, Self::Error> {
        // https://github.com/rustwasm/gloo/blob/master/examples/file-hash/src/lib.rs#L53
        let web_file: &web_sys::File = self.file_handle.as_ref();
        let mut s = ReadableStream::from_raw(web_file.stream().unchecked_into()).into_stream();
        let mut res_chunk = Vec::with_capacity(size as usize);
        while let Some(chunk) = s.try_next().await.unwrap() {
            let mut temp = chunk.unchecked_into::<Uint8Array>();
            res_chunk.append(&mut temp.to_vec());
        }
        Ok(res_chunk)
    }

    fn read_stream(&self) -> Result<Box<dyn Read>, Self::Error> {
        let web_file: &web_sys::File = self.file_handle.as_ref();
        let mut s = ReadableStream::from_raw(web_file.stream().unchecked_into()).into_stream();
        todo!()
    }

    fn get_extension(&self) -> Result<String, Self::Error> {
        let extension = self.filename
            .rsplit_once('.')
            .ok_or(WebBucketFileError::NoExtension)?;
        let (_, extension) = extension; // Unwrap the result
        Ok(extension.to_string())
    }

    fn get_mime_type(&self) -> Result<Mime, Self::Error> {
        let mime = mime::Mime::try_from(self.file_handle.raw_mime_type().as_str()).unwrap();
        Ok(mime)
    }
    //Checks the first couple of bytes of the file to get mime type.
    async fn infer_mime_type(&self) -> Result<infer::Type, Self::Error> {
        let buf = self.read_chunk(16, 0).await?;
        let kind = infer::get(&buf);
        match kind {
            None => Err(WebBucketFileError::UnknownFileType),
            Some(kind) => Ok(kind),
        }
    }

    fn write_chunk(&self, chunk: &vec::Vec<u8>, offset: u64) -> Result<(), Self::Error> {
        let web_file: &web_sys::File = self.file_handle.as_ref();
        let mut writable_stream = WritableStream::from_raw(web_file.stream().unchecked_into()).into_stream();
        let mut writer = writable_stream.get_writer();

        let array = Uint8Array::new_with_length(chunk.len() as u32);
        array.copy_from(chunk.as_slice());

        // Writing the chunk to the stream
        writer.write(&array.to_vec());
        Ok(())
    }

    fn write_stream(&self, stream: &dyn Write) -> Result<(), Self::Error> {
        let web_file: &web_sys::File = self.file_handle.as_ref();
        let mut writable_stream = WritableStream::from_raw(web_file.stream().unchecked_into()).into_stream();
        let mut writer = writable_stream.get_writer();

        todo!()
    }

    fn get_size(&self) -> u64 {
        self.file_handle.size()
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use wasm_bindgen::JsValue;
    use web_sys::{js_sys, Document, Element, HtmlElement, HtmlInputElement};

    fn test_create_file() {
        let file_handle = gloo::file::File::new_with_options(
            &"test.txt",
            "hello world from test file",
            Some(mime::TEXT.into()),
            None,
        );
    }

    // Would have to use trunk test which is still under development https://github.com/trunk-rs/trunk/issues/20
    fn test_write_to_web_file() {
        // https://stackoverflow.com/questions/76855488/no-click-method-defined-on-element-when-created-via-web-sys 
        let document: Document = web_sys::window().unwrap().document().unwrap();
        document.create_element("input");
        let input: Element = document.create_element("input").unwrap();
        input.set_attribute("type", "file").unwrap();
        // For debugging purposes, save a reference to a global variable so I can inspect it from the JavaScript console
        let input_jsval: JsValue = input.clone().into();
        js_sys::Reflect::set(&js_sys::global(), &JsValue::from_str("debug_input"), &input_jsval).unwrap();

        // Attempt to click it
        let input_html_element: HtmlElement = input_jsval.clone().into();
        input_html_element.click();
    }
}
