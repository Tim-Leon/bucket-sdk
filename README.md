# Bucket-SDK
[![Rust](https://github.com/Tim-Leon/bucket-sdk/actions/workflows/rust.yml/badge.svg)](https://github.com/Tim-Leon/bucket-sdk/actions/workflows/rust.yml)  
Welcome to the BucketDrive client repository! This centralized library houses the core code for our secure cloud file storage solution. It includes essential components such as file encryption protocol and Protobuf definitions, along with client.

With this comprehensive library, developers can easily integrate BucketDrive's features into CLI tools and websites. Whether you're building a command-line interface or a user-friendly web application, this client library provides everything you need to interact with our cloud file storage platform seamlessly.

Explore our codebase and leverage the powerful functionalities offered by BucketDrive for secure and efficient file storage in the cloud. Join our community and contribute to the future of cloud-based file management! Please refere to https://docs.bucketdrive.co for more details.

## Features
- Native API (GRPC)
- S3
- Zero Knowledge Encryption
- Signature Validation
- Virtual Filesystem (Under Development)
- 


# 

- api: Higher level api. This is what developers should use. 
- dto: Parsing for parameter when using the higher level api.
- wrapper: Raw request api. 
- client: The different clients supported. currently request or gloo.
- 

## TODO: 
- Add logging for information gathering
- fix upload and download code.
- fix up the API a bit.
- make all enpoint callable.
- Add suport for virtual filesystem.
- test
- mock
- make sure backend and client work in harmony
- TODO: Instead of using type to decide which filestore to use, use trait and have the implementaion accept any struct that implements that trait. 

Each upload and download creates an upload-/download-handler. 
The upload-/download-handler includes, compression, filehandle,  

this handler is a trait that already have implementation for filesystem, other implementations will support AWS, Google cloud and other sources to upload from and support for zero knowledge encryption. 

BucketFileDownloadHandler
BucketFileUploadHandler

There are 4 types of modules:
- Compress Module
- Decompress Module
- Encrypt Module
- Decrypt Module

Each module is defined as a trait, a developer can implement other algorithms over theses traits if needed. 



The upload process starts with a protobuf request asking to upload X files, with the number of bytes, this amount of storage is guaranteed for the duration of the upload to be available to the user.  
after the successful request, a response is set containing a list of URLs that correspond to a file upload. The bytes are then sent to the URL for each file, note that the size can not be changed and it's up to the client to fill the compleate request. 
the upload support being parallel. 


# Navigation Url's

- "bucketdrive.co/{user_id}/{bucket_id}"

### Download

- "eu-central-1-1.bucketdrive.co/download/{user_id}/{bucket_id}/paths"
- "eu-central-1-1.bucketdrive.co/download/{user_id}/{bucket_id}/path"
- "eu-central-1-1.bucketdrive.co/download/{user_id}/{bucket_id}/path"

### Upload

- "bucketdrive.co/upload/{user_id}/{bucket_id}"
- "bucketdrive.co/upload/{user_id}/{bucket_id}/path"