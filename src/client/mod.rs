#[cfg(target_arch = "wasm32")]
mod wasm_client;

#[cfg(not(target_arch = "wasm32"))]
mod native_client;

#[cfg(target_arch = "wasm32")]
pub use wasm_client::*;

#[cfg(not(target_arch = "wasm32"))]
pub use native_client::*;
