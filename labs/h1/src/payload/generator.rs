use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use sha2::{Digest, Sha256};

use super::size::Size;

pub struct Payload {
    pub payload: Vec<u8>,
    hash: Sha256,
}

impl Payload {
    pub fn new(size: &Size, seed: &u64) -> Self {
        let size_bytes = size.to_bytes();
        let mut rng = StdRng::seed_from_u64(*seed);
        let mut data: Vec<u8> = vec![0u8; size_bytes];

        rng.fill_bytes(&mut data);
        let mut hash = Sha256::new();
        hash.update(&data);

        Self {
            payload: data,
            hash,
        }
    }

    pub fn empty(size: Size) -> Self {
        Self {
            payload: Vec::with_capacity(size.to_bytes()),
            hash: Sha256::new(),
        }
    }

    pub fn extend_from_bytes(&mut self, bytes: &[u8]) {
        self.hash.update(bytes);
        self.payload.extend_from_slice(bytes);
    }

    pub fn hash(&self) -> String {
        let hash_string = format!("{:x}", self.hash.clone().finalize());
        hash_string
    }

    pub fn chunks(&self, chunk_size: usize) -> impl Iterator<Item = &[u8]> {
        self.payload.chunks(chunk_size)
    }
}
