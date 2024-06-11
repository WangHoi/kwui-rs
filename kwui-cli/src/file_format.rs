use num_enum::{TryFromPrimitive, IntoPrimitive};

pub const FILE_MAGIC: [u8; 4] = ['K' as _, 'A' as _, 'r' as _, ' ' as _];
pub const FILE_VERSION: u16 = 1;

pub struct Header {
    pub magic: [u8; 4],
    pub version: u16,
    pub flags: u16,
    pub chunk_size: usize,
    pub dir_count: usize,
    pub file_count: usize,
    pub nodes: Vec<Node>,
    pub items: Vec<Item>,
    pub chunks: Vec<Chunk>,
}

#[derive(Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum FileFlag {
    Solid = 2,
}

#[derive(Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum ItemFlag {
    Dir = 1,
}

#[derive(Clone)]
pub struct Node {
    pub ch: u16,
    pub lokid: u16,
    pub eqkid: u16,
    pub hikid: u16,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum AlgorithmType {
    Store,
    Lzf,
}

#[derive(Clone)]
pub struct Chunk {
    pub algorithm: AlgorithmType,
    pub flags: u16,
    pub length: usize,
    pub compressed_length: usize,
}

#[derive(Clone)]
pub struct Item {
    pub digest: [u8; 20], // sha-1
    pub fname: String,
    pub reference: u16,
    pub flags: u16,
    pub offset: usize,
    pub length: usize,
}
