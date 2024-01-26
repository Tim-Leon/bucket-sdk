// use std::sync::{Arc, Mutex};
// use gloo::worker::{HandlerId, Worker, WorkerScope};
//
// pub struct WebFileHandler {
//     pub file: Arc<Mutex<gloo::file::File>>,
// }
//
//
// trait WebFileHandlerTrait : Sync + Send {
//     fn write_chunk(&self,chunk: Vec<u8>) -> Result<>;
//     fn read_chunk(&self, chunk: Vec<u8>) -> Result<>;
//     fn get_size(&self) -> Result<u64>;
//
//
//
// }
//
// impl Worker for WebFileHandler {
//     type Message = ();
//     type Input = ();
//     type Output = ();
//
//     fn create(scope: &WorkerScope<Self>) -> Self {
//         todo!()
//     }
//
//     fn update(&mut self, scope: &WorkerScope<Self>, msg: Self::Message) {
//         todo!()
//     }
//
//     fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, id: HandlerId) {
//         todo!()
//     }
// }
