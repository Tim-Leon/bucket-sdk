use core::slice::SlicePattern;
use generic_array::{ArrayLength, GenericArray};
use secrecy::{ExposeSecret, Secret, Zeroize};

pub mod derived_key;
pub mod master_key;

pub trait EncryptionDeriveKey {
    fn create_key_from();
}

#[derive(Zeroize)]
pub struct SecureGenericArray<T, N: ArrayLength>(pub Secret<GenericArray<T, N>>) where generic_array::GenericArray<T, N>: Zeroize;

impl<T,N: ArrayLength> SlicePattern for SecureGenericArray<T,N> where generic_array::GenericArray<T, N>: Zeroize {
    type Item = T;

    fn as_slice(&self) -> &[Self::Item] {
        self.0.expose_secret().as_slice()
    }
}

//#[derive(Zeroize)]
//pub struct ZeroizeGenericArray<T, N: ArrayLength>(pub GenericArray<T, N>);
