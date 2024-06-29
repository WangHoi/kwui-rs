use crate::file_format::*;
use anyhow;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use lzf::{compress as lzf_compress, decompress as lzf_decompress};
use sha1::{Digest, Sha1};
use size::Size;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

const SOLID_CHUNK_SIZE: usize = 256 << 20;

#[derive(Clone)]
pub struct PackItem {
    pub src: String,
    pub dst: String,
}

pub fn pack(
    output_file: &str,
    file_items: Vec<PackItem>,
    dir_items: Vec<String>,
) -> anyhow::Result<()> {
    let mut f = File::create(output_file)?;

    let header = write_header_data(&mut f, false, file_items.clone(), dir_items.clone())?;

    let mut uncompressed_buffer: Vec<u8> = Vec::new();
    let mut filler = BufferFiller::new(header.items.clone());
    let mut chunks: Vec<Chunk> = Vec::with_capacity(header.items.len());
    for item in header.items.iter() {
        if (item.flags & u16::from(ItemFlag::Dir)) != 0 {
            continue;
        }
        uncompressed_buffer.resize(item.length as usize, 0);
        let uncompressed_size = filler.fill(&mut uncompressed_buffer);
        if uncompressed_size != uncompressed_buffer.len() {
            anyhow::bail!("BufferFiller::fill error.");
        }
        let mut compressed = false;
        if need_compress(&item.fname) {
            match lzf_compress(&uncompressed_buffer[0..uncompressed_size]) {
                Ok(compressed_buffer) => {
                    if compressed_buffer.len() < uncompressed_size * 4 / 5 {
                        println!(
                            "Compress chunk [{}]->[{}]",
                            Size::from_bytes(uncompressed_size),
                            Size::from_bytes(compressed_buffer.len()),
                        );
                        chunks.push(Chunk {
                            length: uncompressed_size,
                            compressed_length: compressed_buffer.len(),
                            algorithm: AlgorithmType::Lzf,
                            flags: 0,
                        });
                        f.write_all(&compressed_buffer)?;
                        compressed = true;
                    }
                }
                Err(e) => {
                    println!("Compress warning [{}], {}", item.fname, e);
                }
            }
        }
        if !compressed {
            chunks.push(Chunk {
                length: uncompressed_size,
                compressed_length: uncompressed_size,
                algorithm: AlgorithmType::Store,
                flags: 0,
            });
            f.write_all(&uncompressed_buffer[0..uncompressed_size])?;
        }
    }
    let (new_total_size, total_compressed) = chunks.iter().fold((0, 0), |acc, x| {
        (acc.0 + x.length, acc.1 + x.compressed_length)
    });

    if header.total_size != new_total_size {
        anyhow::bail!(
            "total_size mismatch {} / {}",
            header.total_size,
            new_total_size
        );
    }
    if header.total_chunks != chunks.len() {
        anyhow::bail!(
            "total_chunks mismatch {} / {}",
            header.total_chunks,
            chunks.len()
        );
    }

    println!(
        "Total compressed [{}] -> [{}]",
        Size::from_bytes(header.total_size),
        Size::from_bytes(total_compressed),
    );

    update_header_data(&mut f, &header, &chunks)?;
    f.flush()?;

    Ok(())
}

struct BufferFiller {
    items: Vec<Item>,
    curr_index: usize,
    curr_offset: usize,
    curr_file: Option<File>,
}

impl BufferFiller {
    fn new(items: Vec<Item>) -> Self {
        Self {
            items,
            curr_index: 0,
            curr_offset: 0,
            curr_file: None,
        }
    }
    fn fill(&mut self, buf: &mut [u8]) -> usize {
        let mut start = 0;
        let end = buf.len();
        while start < end {
            self.advance();
            if self.curr_index >= self.items.len() {
                break;
            }
            if self.curr_file.is_none() {
                break;
            }
            if let Some(ref mut f) = self.curr_file {
                if let Ok(n) = f.read(&mut buf[start..]) {
                    start += n;
                    self.curr_offset += n;
                    if n == 0 {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        start
    }
    fn advance(&mut self) {
        while self.curr_index < self.items.len()
            && self.curr_offset >= self.items[self.curr_index].length as usize
        {
            self.curr_file = None;
            self.curr_offset = 0;
            self.curr_index += 1;
        }
        if self.curr_index < self.items.len() {
            if self.curr_file.is_none() {
                self.curr_file
                    .replace(File::open(&self.items[self.curr_index].fname).unwrap());
            }
        }
    }
}

struct Entry {
    src: String,
    dst: String,
    item_index: usize,
}

struct LookupTable {
    nodes: Vec<Node>,
}

impl Entry {
    fn is_dir(&self) -> bool {
        self.item_index == 0
    }
}

impl Node {
    fn new() -> Self {
        Self {
            ch: 0,
            lokid: u16::MAX,
            eqkid: u16::MAX,
            hikid: u16::MAX,
        }
    }
}

enum ParentNid {
    None,
    Lokid(usize),
    Eqkid(usize),
    Hikid(usize),
}

impl ParentNid {
    fn update(&self, nodes: &mut [Node], nid: usize) {
        match self {
            ParentNid::None => (),
            ParentNid::Lokid(pnid) => nodes[*pnid].lokid = nid as _,
            ParentNid::Eqkid(pnid) => nodes[*pnid].eqkid = nid as _,
            ParentNid::Hikid(pnid) => nodes[*pnid].hikid = nid as _,
        }
    }
}

struct LookupTableIterator {
    index: usize,
    items: Vec<(String, u16)>,
}

impl Iterator for LookupTableIterator {
    type Item = (String, u16);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.items.len() {
            return None;
        }
        self.index += 1;
        Some(self.items[self.index - 1].clone())
    }
}

impl LookupTable {
    fn new() -> Self {
        Self { nodes: Vec::new() }
    }
    fn iter(&self) -> LookupTableIterator {
        let mut items = Vec::<(String, u16)>::new();
        let mut stack = Vec::<ParentNid>::new();
        self.build_items(&mut items, &mut stack, 0);
        LookupTableIterator { index: 0, items }
    }
    fn build_items(
        &self,
        items: &mut Vec<(String, u16)>,
        stack: &mut Vec<ParentNid>,
        current: usize,
    ) {
        if current >= self.nodes.len() {
            return;
        }
        //println!("build_items current={}", current);
        if self.nodes[current].ch == 0 {
            let mut u16str = Vec::<u16>::with_capacity(stack.len());
            for pnid in stack.iter() {
                match pnid {
                    ParentNid::None => (),
                    ParentNid::Lokid(_) => (),
                    ParentNid::Eqkid(idx) => u16str.push(self.nodes[*idx].ch),
                    ParentNid::Hikid(_) => (),
                }
            }
            let key = String::from_utf16_lossy(&u16str);
            let value = self.nodes[current].eqkid;
            items.push((key, value));
        }
        stack.push(ParentNid::Lokid(current));
        self.build_items(items, stack, self.nodes[current].lokid as _);
        stack.pop();

        if self.nodes[current].ch != 0 {
            stack.push(ParentNid::Eqkid(current));
            self.build_items(items, stack, self.nodes[current].eqkid as _);
            stack.pop();
        }

        stack.push(ParentNid::Hikid(current));
        self.build_items(items, stack, self.nodes[current].hikid as _);
        stack.pop();
    }
    fn insert(&mut self, s: &str, v: u16) -> u16 {
        let mut s = s.encode_utf16().collect::<Vec<_>>();
        s.push(0);

        let mut sidx = 0usize;
        let mut nid = 0usize;
        let mut pnid = ParentNid::None;
        while nid < self.nodes.len() {
            let sch = s[sidx];
            let node = &self.nodes[nid];
            if sch < node.ch {
                pnid = ParentNid::Lokid(nid);
                nid = node.lokid as _;
            } else if sch > node.ch {
                pnid = ParentNid::Hikid(nid);
                nid = node.hikid as _;
            } else {
                pnid = ParentNid::Eqkid(nid);
                nid = node.eqkid as _;
                sidx += 1;
                if sch == 0 {
                    return node.eqkid;
                }
            }
        }

        for sch in &s[sidx..] {
            let nid = self.nodes.len();
            pnid.update(&mut self.nodes, nid);

            let mut node = Node::new();
            node.ch = *sch;
            if *sch == 0 {
                node.eqkid = v;
            }
            self.nodes.push(node);
            pnid = ParentNid::Eqkid(nid);
        }
        0
    }
}

fn build_entry_and_item(
    mut file_items: Vec<PackItem>,
    dir_items: Vec<String>,
) -> anyhow::Result<(Vec<Entry>, Vec<Item>, usize)> {
    let mut entries: Vec<Entry> = Vec::new();
    let mut items: Vec<Item> = Vec::new();

    // process dir_items
    items.push(Item {
        fname: String::new(),
        digest: empty_sha1(),
        reference: dir_items.len() as _,
        flags: ItemFlag::Dir as _,
        offset: 0,
        length: 0,
    });
    for d in dir_items.iter() {
        entries.push(Entry {
            src: d.clone(),
            dst: d.clone(),
            item_index: 0,
        });
    }

    // process file_items

    // sort for solid
    file_items.sort_by_key(solid_sort_key);

    // dedup
    let mut total_dups = 0;
    let mut total_file_items = 0;
    let mut offset = 0;
    for f in file_items.iter() {
        println!("pack: scan_file {}", f.src);
        let (length, digest) = scan_file(&f.src)?;
        let item_idx = find_item(&items, length as _, &digest);
        if let Some(item_idx) = item_idx {
            items[item_idx].reference += 1;
            total_dups += 1;
            println!("find dup [{}] and [{}]", items[item_idx].fname, f.src);
            entries.push(Entry {
                src: f.src.clone(),
                dst: f.dst.clone(),
                item_index: item_idx,
            });
        } else {
            items.push(Item {
                fname: f.src.clone(),
                digest,
                reference: 1,
                flags: 0,
                offset,
                length: length as usize,
            });
            entries.push(Entry {
                src: f.src.clone(),
                dst: f.dst.clone(),
                item_index: items.len() - 1,
            });
            offset += length as usize;
            total_file_items += 1;
        }
    }
    println!("total_dups {}", total_dups);
    Ok((entries, items, total_file_items))
}

fn empty_sha1() -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(b"");
    hasher.finalize().into()
}

fn find_item(items: &[Item], length: usize, digest: &[u8; 20]) -> Option<usize> {
    for (idx, item) in items.iter().enumerate() {
        if idx == 0 {
            continue;
        }
        if item.length == length && item.digest == *digest {
            return Some(idx);
        }
    }
    None
}

fn scan_file(fpath: &str) -> anyhow::Result<(u64, [u8; 20])> {
    let f = File::open(fpath)?;

    #[cfg(windows)]
    let file_len = f.metadata()?.file_size();
    #[cfg(unix)]
    let file_len = f.metadata()?.size();

    let mut reader = BufReader::new(f);
    let mut hasher = Sha1::new();
    loop {
        let length = {
            let buffer = reader.fill_buf()?;
            hasher.update(buffer);
            buffer.len()
        };
        if length == 0 {
            break;
        }
        reader.consume(length);
    }
    let digest = hasher.finalize().into();
    Ok((file_len, digest))
}

fn solid_sort_key(p: &PackItem) -> (i32, String, String, String) {
    let src = p.src.to_lowercase();
    let mut prio: i32 = 0;
    let path = PathBuf::from(&src);
    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    if let Some(idx) = src.rfind(".") {
        let suffix = &src[(idx + 1)..];
        if suffix == "exe" || suffix == "dll" || suffix == "ocx" || suffix == "sys" {
            prio = -1;
        }
        (prio, suffix.to_string(), file_name, src)
    } else {
        (prio, "zzz".to_string(), file_name, src)
    }
}

pub fn unpack(input_file: String, target_dir: PathBuf) -> anyhow::Result<()> {
    let mut f = File::open(&input_file)?;
    let header = read_header(&mut f)?;

    let mut lookup_table = LookupTable::new();
    lookup_table.nodes = header.nodes.clone();

    println!("dir_count={} file_count={}", header.dir_count, header.file_count);
    println!("node_count={} item_count={} chunk_count={}",
        header.nodes.len(), header.items.len(), header.chunks.len());

    let mut work_items = lookup_table.iter().collect::<Vec<_>>();
    work_items.sort_by_key(|(_k, idx)| *idx);
    //println!("work_item_count={}", work_items.len());

    // (item_index, dst)
    let mut last_item: Option<(u16, PathBuf)> = None;
    let mut bw = BufferWriter::new(&mut f, header.chunk_size, header.chunks);
    let mut offset = 0usize;
    for (dst_path, item_idx) in work_items.iter() {
        println!("extract [{}]...", dst_path);
        let dst = target_dir.join(&dst_path[1..]);
        if *item_idx == 0 {
            std::fs::create_dir_all(dst)?;
        } else {
            if let Some((last_idx, last_dst)) = last_item.as_ref() {
                if last_idx == item_idx {
                    println!(
                        "extract dup [{}]->[{}]",
                        last_dst.to_string_lossy(),
                        dst.to_string_lossy()
                    );
                    std::fs::copy(last_dst, dst)?;
                    continue;
                }
            }
            let mut f = File::create(&dst)?;
            let item_offset = header.items[*item_idx as usize].offset as usize;
            let length = header.items[*item_idx as usize].length as usize;
            // println!("off={} offset={}", off, offset);
            bw.write(&mut f, length)?;
            offset += length;
            last_item.replace((*item_idx, dst));
        }
    }

    Ok(())
}

pub fn list(input_file: String) -> anyhow::Result<()> {
    let mut f = File::open(&input_file)?;
    let header = read_header(&mut f)?;

    let mut lookup_table = LookupTable::new();
    lookup_table.nodes = header.nodes.clone();

    if (header.flags & u16::from(FileFlag::Solid)) != 0 {
        print!("solid archive ");
    } else {
        print!("archive ");
    }
    println!("dir_count={} file_count={}", header.dir_count, header.file_count);
    println!("node_count={} item_count={} chunk_count={}",
        header.nodes.len(), header.items.len(), header.chunks.len());

    let mut work_items = lookup_table.iter().collect::<Vec<_>>();
    work_items.sort_by_key(|(_k, idx)| *idx);
    //println!("work_item_count={}", work_items.len());

    for (i, (dst_path, _item_idx)) in work_items.iter().enumerate() {
        println!("#{}: {}", i + 1, dst_path);
    }

    Ok(())
}
struct BufferWriter<'r, R: std::io::Read> {
    reader: &'r mut R,
    chunk_size: usize,
    chunks: Vec<Chunk>,
    curr_index: usize,
    curr_offset: usize,
    curr_buf: Vec<u8>,
    fetched: bool,
}

impl<'r, R: std::io::Read> BufferWriter<'r, R> {
    fn new(reader: &'r mut R, chunk_size: usize, chunks: Vec<Chunk>) -> Self {
        Self {
            reader,
            chunk_size,
            chunks,
            curr_index: 0,
            curr_offset: 0,
            curr_buf: Vec::with_capacity(chunk_size),
            fetched: false,
        }
    }
    fn write<W: std::io::Write>(
        &mut self,
        writer: &mut W,
        length: usize,
    ) -> std::io::Result<usize> {
        let mut remain = length;
        while remain > 0 {
            self.fetch()?;
            let n = std::cmp::min(remain, self.curr_buf.len() - self.curr_offset);
            if n == 0 {
                return Err(std::io::ErrorKind::InvalidData.into());
            }
            writer.write_all(&self.curr_buf[self.curr_offset..(self.curr_offset + n)])?;
            self.curr_offset += n;
            remain -= n;
        }
        Ok(length)
    }
    fn fetch_chunk(&mut self, chunk: Chunk) -> std::io::Result<()> {
        self.curr_buf.resize(chunk.length, 0);
        let mut compressed = Vec::<u8>::with_capacity(chunk.compressed_length);
        compressed.resize(chunk.compressed_length, 0);
        self.reader.read(&mut compressed)?;
        //println!("fetch_chunk algo={} size {} <- {}", chunk.algorithm as u16, chunk.length, chunk.compressed_length);

        if chunk.algorithm == AlgorithmType::Store {
            self.curr_buf.copy_from_slice(&compressed);
        } else if chunk.algorithm == AlgorithmType::Lzf {
            self.curr_buf =
                lzf_decompress(&compressed, self.curr_buf.len()).map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("lzf_decompress error {}", e),
                    )
                })?;
        } else {
            return Err(std::io::Error::other("invalid chunk compression algorithm"));
        }
        Ok(())
    }
    fn fetch(&mut self) -> std::io::Result<()> {
        if !self.fetched {
            self.fetched = true;
            if let Some(chunk) = self.chunks.first() {
                self.fetch_chunk(chunk.clone())?;
            }
        }
        while self.curr_offset >= self.curr_buf.len() {
            self.curr_index += 1;
            self.curr_offset = 0;
            self.curr_buf.clear();
            if self.curr_index >= self.chunks.len() {
                break;
            }
            self.fetch_chunk(self.chunks[self.curr_index].clone())?;
        }
        Ok(())
    }
}

fn read_header(f: &mut File) -> anyhow::Result<Header> {
    let mut magic = [0u8; 4];
    f.read(&mut magic)?;
    if magic != FILE_MAGIC {
        anyhow::bail!("invalid file magic");
    }
    let version = f.read_u16::<LittleEndian>()?;
    if version != FILE_VERSION {
        anyhow::bail!("invalid file version {}", version);
    }
    let flags = f.read_u16::<LittleEndian>()?;
    let chunk_size = f.read_u32::<LittleEndian>()? as usize;
    let dir_count = f.read_u32::<LittleEndian>()? as usize;
    let file_count = f.read_u32::<LittleEndian>()? as usize;

    let node_count = f.read_u32::<LittleEndian>()? as usize;
    let mut nodes = Vec::with_capacity(node_count);
    for _ in 0..node_count {
        let ch = f.read_u16::<LittleEndian>()?;
        let lokid = f.read_u16::<LittleEndian>()?;
        let eqkid = f.read_u16::<LittleEndian>()?;
        let hikid = f.read_u16::<LittleEndian>()?;
        nodes.push(Node {
            ch,
            lokid,
            eqkid,
            hikid,
        });
    }

    let item_count = f.read_u32::<LittleEndian>()? as usize;
    let mut items = Vec::<Item>::with_capacity(item_count);
    for _ in 0..item_count {
        let reference = f.read_u16::<LittleEndian>()?;
        let flags = f.read_u16::<LittleEndian>()?;
        let offset = f.read_u32::<LittleEndian>()? as _;
        let length = f.read_u32::<LittleEndian>()? as _;
        items.push(Item {
            digest: [0u8; 20],
            fname: String::new(),
            reference,
            flags,
            offset,
            length,
        });
    }

    let chunk_count = f.read_u32::<LittleEndian>()? as usize;
    let mut chunks = Vec::<Chunk>::with_capacity(chunk_count);
    for _ in 0..chunk_count {
        let algorithm = f.read_u16::<LittleEndian>()?.try_into()?;
        let flags = f.read_u16::<LittleEndian>()?;
        let chunk_size = f.read_u32::<LittleEndian>()? as usize;
        let compressed_size = f.read_u32::<LittleEndian>()? as usize;
        chunks.push(Chunk {
            algorithm,
            flags,
            length: chunk_size,
            compressed_length: compressed_size,
        });
    }
    
    Ok(Header {
        magic,
        version,
        flags,
        chunk_size,
        dir_count,
        file_count,
        nodes,
        items,
        chunks,
    })
}

struct HeaderData {
    total_size: usize,
    total_chunks: usize,
    items: Vec<Item>,
    nodes_file_offset: u64,
    items_file_offset: u64,
    chunks_file_offset: u64,
    data_file_offset: u64,
}

fn write_header_data(
    f: &mut File,
    solid: bool,
    file_items: Vec<PackItem>,
    dir_items: Vec<String>,
) -> anyhow::Result<HeaderData> {
    let chunk_size: usize = if solid { SOLID_CHUNK_SIZE } else { 0 };
    f.write(&FILE_MAGIC)?;
    f.write_u16::<LittleEndian>(FILE_VERSION)?;
    let file_flags: u16 = if solid { FileFlag::Solid.into() } else { 0 };
    f.write_u16::<LittleEndian>(file_flags)?;
    f.write_u32::<LittleEndian>(chunk_size as _)?;
    f.write_u32::<LittleEndian>(dir_items.len() as _)?;
    f.write_u32::<LittleEndian>(file_items.len() as _)?;

    let (entries, items, num_file_items) = build_entry_and_item(file_items, dir_items)?;
    let mut tbl = LookupTable::new();
    for e in entries.iter() {
        tbl.insert(&e.dst, e.item_index as _);
    }
    println!("lookupTbl len={}", tbl.nodes.len());
    let nodes_file_offset = f.seek(SeekFrom::Current(0)).unwrap();
    f.write_u32::<LittleEndian>(tbl.nodes.len() as u32)?;
    for node in tbl.nodes.iter() {
        f.write_u16::<LittleEndian>(node.ch)?;
        f.write_u16::<LittleEndian>(node.lokid)?;
        f.write_u16::<LittleEndian>(node.eqkid)?;
        f.write_u16::<LittleEndian>(node.hikid)?;
    }

    println!("item len={}", items.len());
    let items_file_offset = f.seek(SeekFrom::Current(0)).unwrap();
    f.write_u32::<LittleEndian>(items.len() as u32)?;
    for item in items.iter() {
        f.write_u16::<LittleEndian>(item.reference)?;
        f.write_u16::<LittleEndian>(item.flags)?;
        f.write_u32::<LittleEndian>(item.offset as _)?;
        f.write_u32::<LittleEndian>(item.length as _)?;
    }

    let total_size = items.iter().fold(0, |acc, x| acc + x.length);
    let total_chunks = if solid {
        (total_size as usize + SOLID_CHUNK_SIZE - 1) / SOLID_CHUNK_SIZE
    } else {
        num_file_items
    };
    let mut chunks: Vec<Chunk> = Vec::with_capacity(total_chunks);
    chunks.resize(
        total_chunks,
        Chunk {
            algorithm: AlgorithmType::Store,
            flags: 0,
            length: 0,
            compressed_length: 0,
        },
    );
    println!("chunkTbl len={}", chunks.len());
    let chunks_file_offset = f.seek(SeekFrom::Current(0)).unwrap();
    f.write_u32::<LittleEndian>(chunks.len() as u32)?;
    for chunk in chunks.iter() {
        f.write_u16::<LittleEndian>(chunk.algorithm as _)?;
        f.write_u16::<LittleEndian>(chunk.flags)?;
        f.write_u32::<LittleEndian>(chunk.length as _)?;
        f.write_u32::<LittleEndian>(chunk.compressed_length as _)?;
    }

    let data_file_offset = f.seek(SeekFrom::Current(0)).unwrap();

    Ok(HeaderData {
        total_size: total_size as _,
        total_chunks,
        items,
        nodes_file_offset,
        items_file_offset,
        chunks_file_offset,
        data_file_offset,
    })
}

fn update_header_data(f: &mut File, header: &HeaderData, chunks: &[Chunk]) -> anyhow::Result<()> {
    f.seek(SeekFrom::Start(header.chunks_file_offset))?;
    f.write_u32::<LittleEndian>(chunks.len() as u32)?;
    for chunk in chunks.iter() {
        // println!("algo={}", chunk.algorithm as u16);
        f.write_u16::<LittleEndian>(chunk.algorithm as _)?;
        f.write_u16::<LittleEndian>(chunk.flags)?;
        f.write_u32::<LittleEndian>(chunk.length as _)?;
        f.write_u32::<LittleEndian>(chunk.compressed_length as _)?;
    }

    Ok(())
}

fn need_compress(fname: &str) -> bool {
    let fname = fname.to_lowercase();
    let is_image = fname.ends_with(".png")
        || fname.ends_with(".gif")
        || fname.ends_with(".jpg")
        || fname.ends_with(".jpeg");
    !is_image
}
