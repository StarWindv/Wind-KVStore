// src/lib.rs
use anyhow::{anyhow, bail, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use linked_hash_map::LinkedHashMap;
use log::error;
use memmap2::{MmapMut, MmapOptions};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{Cursor, Read, Write};
use std::mem;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

// 常量定义
const MAGIC_NUMBER: u32 = 0x4B5644; // 'KVD' in ASCII
const PAGE_SIZE: usize = 1024; // 1KB 页大小
const HEADER_SIZE: usize = 128; // 文件头大小
const MAX_CACHE_SIZE: usize = 100 * 1024; // 100KB 缓存
const OVERFLOW_THRESHOLD: usize = 900; // 溢出阈值
const WAL_FILE_EXT: &str = ".wal"; // WAL文件后缀


#[derive(Debug, Error)]
pub enum KvError {
    #[error("Invalid magic number")]
    InvalidMagic,
    #[error("Invalid page header")]
    InvalidPageHeader,
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Other error: {0}")]
    #[allow(dead_code)]
    Other(String),
}


// 分页头结构
#[derive(Debug, Clone, Copy)]
struct PageHeader {
    flags: u8,      // 状态标志
    kv_count: u16,  // 键值对数量
    data_len: u16,  // 数据长度
    next_page: u32, // 下一页号
}


impl PageHeader {
    const SIZE: usize = 16;

    fn pack(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        let mut cursor = Cursor::new(&mut buf[..]);
        cursor.write_u8(self.flags).unwrap();
        cursor.write_u16::<LittleEndian>(self.kv_count).unwrap();
        cursor.write_u16::<LittleEndian>(self.data_len).unwrap();
        cursor.write_u32::<LittleEndian>(self.next_page).unwrap();
        cursor.write_all(&[0u8; 7]).unwrap(); // 保留区
        buf
    }

    fn unpack(data: &[u8]) -> Result<Self> {
        if data.len() < Self::SIZE {
            bail!(KvError::InvalidPageHeader);
        }

        let mut cursor = Cursor::new(data);
        let flags = cursor.read_u8()?;
        let kv_count = cursor.read_u16::<LittleEndian>()?;
        let data_len = cursor.read_u16::<LittleEndian>()?;
        let next_page = cursor.read_u32::<LittleEndian>()?;
        cursor.read_exact(&mut [0u8; 7])?; // 跳过保留区

        Ok(Self {
            flags,
            kv_count,
            data_len,
            next_page,
        })
    }
}


// 数据库文件头结构
#[derive(Debug)]
struct DBHeader {
    magic: u32,                 // 魔数
    db_identifier: String,      // 数据库标识
    create_time: u64,           // 创建时间（毫秒）
    modify_time: u64,           // 修改时间（毫秒）
    page_size: u16,             // 页大小
    total_pages: u32,           // 总页数
    overflow_start: u32,        // 溢出页起始
    free_page_head: u32,        // 空闲页链表头
}


impl DBHeader {
    const SIZE: usize = HEADER_SIZE;

    fn pack(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        let mut cursor = Cursor::new(&mut buf[..]);

        // 确保标识符不超过31字节
        let mut identifier = self.db_identifier.clone();
        if identifier.len() > 31 {
            identifier.truncate(31);
        }
        let identifier_bytes = identifier.as_bytes();
        let mut padded_identifier = [0u8; 32];
        padded_identifier[..identifier_bytes.len()].copy_from_slice(identifier_bytes);

        cursor.write_u32::<LittleEndian>(self.magic).unwrap();
        cursor.write_all(&padded_identifier).unwrap();
        cursor.write_u64::<LittleEndian>(self.create_time).unwrap();
        cursor.write_u64::<LittleEndian>(self.modify_time).unwrap();
        cursor.write_u16::<LittleEndian>(self.page_size).unwrap();
        cursor.write_u32::<LittleEndian>(self.total_pages).unwrap();
        cursor.write_u32::<LittleEndian>(self.overflow_start).unwrap();
        cursor.write_u32::<LittleEndian>(self.free_page_head).unwrap();
        cursor.write_all(&[0u8; 62]).unwrap(); // 保留区

        buf
    }

    fn unpack(data: &[u8]) -> Result<Self> {
        if data.len() < Self::SIZE {
            bail!("Header data too short");
        }

        let mut cursor = Cursor::new(data);
        let magic = cursor.read_u32::<LittleEndian>()?;
        if magic != MAGIC_NUMBER {
            bail!(KvError::InvalidMagic);
        }

        let mut identifier = [0u8; 32];
        cursor.read_exact(&mut identifier)?;
        let identifier_len = identifier.iter().position(|&b| b == 0).unwrap_or(32);
        let db_identifier = String::from_utf8_lossy(&identifier[..identifier_len]).to_string();

        let create_time = cursor.read_u64::<LittleEndian>()?;
        let modify_time = cursor.read_u64::<LittleEndian>()?;
        let page_size = cursor.read_u16::<LittleEndian>()?;
        let total_pages = cursor.read_u32::<LittleEndian>()?;
        let overflow_start = cursor.read_u32::<LittleEndian>()?;
        let free_page_head = cursor.read_u32::<LittleEndian>()?;
        cursor.read_exact(&mut [0u8; 62])?; // 跳过保留区

        Ok(Self {
            magic,
            db_identifier,
            create_time,
            modify_time,
            page_size,
            total_pages,
            overflow_start,
            free_page_head,
        })
    }
}


// LRU缓存实现
struct LRUCache {
    cache: LinkedHashMap<u32, Vec<u8>>,
    max_size: usize,
    curr_size: usize,
}


impl LRUCache {
    fn new(max_size: usize) -> Self {
        Self {
            cache: LinkedHashMap::new(),
            max_size,
            curr_size: 0,
        }
    }

    fn get(&mut self, key: u32) -> Option<Vec<u8>> {
        self.cache.get_refresh(&key).cloned()
    }

    fn put(&mut self, key: u32, value: Vec<u8>) {
        let value_size = value.len();
        if value_size > self.max_size {
            return;
        }

        if let Some(old_value) = self.cache.insert(key, value) {
            self.curr_size -= old_value.len();
        } else {
            while self.curr_size + value_size > self.max_size && !self.cache.is_empty() {
                if let Some((_, evicted)) = self.cache.pop_front() {
                    self.curr_size -= evicted.len();
                }
            }
        }

        self.curr_size += value_size;
    }


    #[allow(dead_code)]
    fn contains_key(&self, key: u32) -> bool {
        self.cache.contains_key(&key)
    }


    #[allow(dead_code)]
    fn remove(&mut self, key: u32) -> Option<Vec<u8>> {
        self.cache.remove(&key).map(|v| {
            self.curr_size -= v.len();
            v
        })
    }


    #[allow(dead_code)]
    fn clear(&mut self) {
        self.cache.clear();
        self.curr_size = 0;
    }
}


// 预写日志管理器
struct WALManager {
    wal_path: PathBuf,
}


impl WALManager {
    const OP_PUT: u8 = 0;
    const OP_DELETE: u8 = 1;

    fn new(db_path: &Path) -> Self {
        let mut wal_path = db_path.to_path_buf();
        wal_path.set_extension(WAL_FILE_EXT);
        Self { wal_path }
    }

    fn log_operation(&self, op_type: u8, key: &[u8], value: Option<&[u8]>) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.wal_path)?;

        file.write_all(&[op_type])?;

        // 写入键长度和键
        file.write_u16::<LittleEndian>(key.len() as u16)?;
        file.write_all(key)?;

        // 写入值长度和值（如果存在）
        let value = value.unwrap_or_default();
        file.write_u32::<LittleEndian>(value.len() as u32)?;
        file.write_all(value)?;

        file.sync_all()?;
        Ok(())
    }

    fn recover(self, store: &mut KVStore) -> Result<()> {
        if !self.wal_path.exists() {
            return Ok(());
        }

        let mut file = File::open(&self.wal_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let mut pos = 0;
        while pos < buffer.len() {
            if pos + 1 > buffer.len() {
                break;
            }
            let op_type = buffer[pos];
            pos += 1;

            if pos + 2 > buffer.len() {
                break;
            }
            let key_len = u16::from_le_bytes([buffer[pos], buffer[pos + 1]]) as usize;
            pos += 2;

            if pos + key_len > buffer.len() {
                break;
            }
            let key = buffer[pos..pos + key_len].to_vec();
            pos += key_len;

            if pos + 4 > buffer.len() {
                break;
            }
            let value_len = u32::from_le_bytes([
                buffer[pos],
                buffer[pos + 1],
                buffer[pos + 2],
                buffer[pos + 3],
            ]) as usize;
            pos += 4;

            if pos + value_len > buffer.len() {
                break;
            }
            let value = buffer[pos..pos + value_len].to_vec();
            pos += value_len;

            match op_type {
                Self::OP_PUT => {
                    if let Err(e) = store.put_internal(&key, &value, false) {
                        error!("WAL recovery put failed: {}", e);
                    }
                }
                Self::OP_DELETE => {
                    if let Err(e) = store.delete_internal(&key, false) {
                        error!("WAL recovery delete failed: {}", e);
                    }
                }
                _ => {
                    error!("Unknown WAL operation type: {}", op_type);
                }
            }
        }

        // 删除WAL文件
        std::fs::remove_file(&self.wal_path)?;
        Ok(())
    }
}


// 键值存储引擎
pub struct KVStore {
    path: PathBuf,
    file: File,
    mmap: MmapMut,
    header: DBHeader,
    key_to_page: std::collections::HashMap<Vec<u8>, u32>,
    page_cache: LRUCache,
    dirty_pages: HashSet<u32>,
    wal_manager: WALManager,
    last_used_page: u32,
}


impl KVStore {
    pub fn open<P: AsRef<Path>>(path: P, db_identifier: Option<&str>) -> Result<Self> {
        let path = path.as_ref();
        let wal_manager = WALManager::new(path);

        if !path.exists() {
            Self::create_new_db(path, db_identifier, wal_manager)
        } else {
            Self::open_existing_db(path, db_identifier, wal_manager)
        }
    }


    fn create_new_db(
        path: &Path,
        db_identifier: Option<&str>,
        wal_manager: WALManager,
    ) -> Result<Self> {
        // 文件需要可变，但变量名不需要 mut
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        let file = file; // 重新绑定为可变
        file.set_len((HEADER_SIZE + PAGE_SIZE) as u64)?;

        let mut mmap = unsafe { MmapOptions::new().map_mut(&file)? };

        let identifier = db_identifier.unwrap_or("KVStore").to_string();
        let now = current_time_millis();
        let header = DBHeader {
            magic: MAGIC_NUMBER,
            db_identifier: identifier,
            create_time: now,
            modify_time: now,
            page_size: PAGE_SIZE as u16,
            total_pages: 1,
            overflow_start: 0,
            free_page_head: 0,
        };

        mmap[..DBHeader::SIZE].copy_from_slice(&header.pack());

        // 初始化第一页
        let page_header = PageHeader {
            flags: 0,
            kv_count: 0,
            data_len: 0,
            next_page: 0,
        };
        mmap[HEADER_SIZE..HEADER_SIZE + PageHeader::SIZE].copy_from_slice(&page_header.pack());

        mmap.flush()?;

        Ok(Self {
            path: path.to_path_buf(),
            file,
            mmap,
            header,
            key_to_page: std::collections::HashMap::new(),
            page_cache: LRUCache::new(MAX_CACHE_SIZE),
            dirty_pages: HashSet::new(),
            wal_manager,
            last_used_page: 1,
        })
    }


    fn open_existing_db(
        path: &Path,
        db_identifier: Option<&str>,
        wal_manager: WALManager,
    ) -> Result<Self> {
        // 文件需要可变，但变量名不需要 mut
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let file = file; // 重新绑定为可变
        let mmap = unsafe { MmapOptions::new().map_mut(&file)? };

        if mmap.len() < HEADER_SIZE {
            bail!("Database file too small");
        }

        let mut header = DBHeader::unpack(&mmap[..HEADER_SIZE])?;
        if let Some(id) = db_identifier {
            header.db_identifier = id.to_string();
        }

        let mut store = Self {
            path: path.to_path_buf(),
            file,
            mmap,
            header,
            key_to_page: std::collections::HashMap::new(),
            page_cache: LRUCache::new(MAX_CACHE_SIZE),
            dirty_pages: HashSet::new(),
            wal_manager,
            last_used_page: 0,
        };

        store.build_index()?;

        // 临时取出 wal_manager 进行恢复操作
        let wal_manager = mem::replace(
            &mut store.wal_manager,
            WALManager::new(&store.path),
        );
        wal_manager.recover(&mut store)?;

        Ok(store)
    }


    pub fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.wal_manager
            .log_operation(WALManager::OP_PUT, key, Some(value))?;
        self.put_internal(key, value, true)
    }


    fn put_internal(&mut self, key: &[u8], value: &[u8], update_index: bool) -> Result<()> {
        if self.key_to_page.contains_key(key) {
            self.update_existing(key, value)?;
            return Ok(());
        }

        let required_space = 1 + key.len() + 2 + value.len();

        if self.last_used_page != 0 {
            let page_data = self.read_page(self.last_used_page)?;
            let header = PageHeader::unpack(&page_data)?;

            let free_space = PAGE_SIZE - PageHeader::SIZE - header.data_len as usize;
            if required_space <= free_space {
                self.insert_to_page(self.last_used_page, key, value, update_index)?;
                return Ok(());
            }
        }

        // 收集缓存的页面号副本
        let cached_pages: Vec<u32> = self.page_cache.cache.keys().cloned().collect();
        for page_num in cached_pages {
            if page_num == self.last_used_page {
                continue;
            }

            let page_data = self.read_page(page_num)?;
            let header = PageHeader::unpack(&page_data)?;

            let free_space = PAGE_SIZE - PageHeader::SIZE - header.data_len as usize;
            if required_space <= free_space {
                self.insert_to_page(page_num, key, value, update_index)?;
                return Ok(());
            }
        }

        // 分配新页
        let new_page = self.allocate_page()?;
        self.insert_to_page(new_page, key, value, update_index)?;
        Ok(())
    }


    pub fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let page_num = match self.key_to_page.get(key) {
            Some(num) => *num,
            None => return Ok(None),
        };

        let page_data = self.read_page(page_num)?;
        let header = PageHeader::unpack(&page_data)?;
        let data_start = PageHeader::SIZE;
        let data_end = data_start + header.data_len as usize;
        let data = &page_data[data_start..data_end];

        let mut pos = 0;
        for _ in 0..header.kv_count {
            if pos >= data.len() {
                break;
            }

            let klen = data[pos] as usize;
            pos += 1;

            if pos + klen > data.len() {
                break;
            }
            let current_key = &data[pos..pos + klen];
            pos += klen;

            if pos + 2 > data.len() {
                break;
            }
            let vlen = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;

            if pos + vlen > data.len() {
                break;
            }

            if current_key == key {
                let mut value = data[pos..pos + vlen].to_vec();

                // 处理溢出
                if header.flags & 0x02 != 0 {
                    let overflow_data = self.read_overflow(header.next_page)?;
                    value.extend_from_slice(&overflow_data);
                }

                return Ok(Some(value));
            }

            pos += vlen;
        }

        Ok(None)
    }


    pub fn delete(&mut self, key: &[u8]) -> Result<()> {
        self.wal_manager
            .log_operation(WALManager::OP_DELETE, key, None)?;
        self.delete_internal(key, true)
    }


    fn delete_internal(&mut self, key: &[u8], update_index: bool) -> Result<()> {
        let page_num = match self.key_to_page.get(key) {
            Some(num) => *num,
            None => return Ok(()),
        };

        let page_data = self.read_page(page_num)?;
        let mut header = PageHeader::unpack(&page_data)?;
        let data_start = PageHeader::SIZE;
        let data_end = data_start + header.data_len as usize;
        let data = &page_data[data_start..data_end];

        let mut pos = 0;
        let mut found = false;
        let mut kv_start = 0;
        for i in 0..header.kv_count {
            kv_start = pos;
            let klen = data[pos] as usize;
            pos += 1;

            if pos + klen > data.len() {
                break;
            }
            let current_key = &data[pos..pos + klen];
            pos += klen;

            if pos + 2 > data.len() {
                break;
            }
            let vlen = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;

            if pos + vlen > data.len() {
                break;
            }

            if current_key == key {
                found = true;
                // 如果是第一个键且有溢出，释放溢出页
                if i == 0 && header.flags & 0x02 != 0 {
                    self.free_overflow(header.next_page)?;
                    header.flags &= !0x02;
                    header.next_page = 0;
                }
                break;
            }

            pos += vlen;
        }

        if !found {
            return Ok(());
        }

        let kv_end = pos;
        let before = &data[..kv_start];
        let after = &data[kv_end..];
        let new_data = [before, after].concat();

        header.kv_count -= 1;
        header.data_len = new_data.len() as u16;

        if update_index {
            self.key_to_page.remove(key);
        }

        // 如果页空了，释放它
        if header.kv_count == 0 {
            self.free_page(page_num)?;
            return Ok(());
        }

        // 更新页数据
        let mut new_page_data = header.pack().to_vec();
        new_page_data.extend_from_slice(&new_data);
        new_page_data.resize(PAGE_SIZE, 0);

        self.write_page(page_num, &new_page_data)?;
        Ok(())
    }


    pub fn compact(&mut self) -> Result<()> {
        let temp_path = self.path.with_extension("tmp");
        let mut temp_db = KVStore::create_new_db(
            &temp_path,
            Some(&self.header.db_identifier),
            WALManager::new(&temp_path),
        )?;

        // 扫描所有键
        for page_num in 1..=self.header.total_pages {
            if self.is_free_page(page_num)? {
                continue;
            }

            let page_data = self.read_page(page_num)?;
            let header = PageHeader::unpack(&page_data)?;
            let data_start = PageHeader::SIZE;
            let data_end = data_start + header.data_len as usize;
            let data = &page_data[data_start..data_end];

            let mut pos = 0;
            for _ in 0..header.kv_count {
                if pos >= data.len() {
                    break;
                }

                let klen = data[pos] as usize;
                pos += 1;

                if pos + klen > data.len() {
                    break;
                }
                let key = &data[pos..pos + klen];
                pos += klen;

                if pos + 2 > data.len() {
                    break;
                }
                let vlen = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
                pos += 2;

                if pos + vlen > data.len() {
                    break;
                }
                let value = &data[pos..pos + vlen];
                pos += vlen;

                if let Ok(Some(existing)) = temp_db.get(key) {
                    // 如果键已存在，更新它
                    temp_db.put(key, &existing)?;
                } else {
                    // 否则插入新键
                    temp_db.put(key, value)?;
                }
            }
        }

        // 关闭当前数据库
        self.flush_pages()?;
        self.update_header()?;
        self.file.sync_all()?;

        // 替换文件
        std::fs::rename(&temp_path, &self.path)?;

        // 重新打开数据库
        *self = KVStore::open_existing_db(
            &self.path,
            Some(&self.header.db_identifier),
            WALManager::new(&self.path),
        )?;
        Ok(())
    }


    pub fn get_identifier(&self) -> &str {
        &self.header.db_identifier
    }


    pub fn set_identifier(&mut self, identifier: &str) -> Result<()> {
        self.header.db_identifier = identifier.to_string();
        self.update_header()
    }


    fn build_index(&mut self) -> Result<()> {
        self.key_to_page.clear();

        for page_num in 1..=self.header.total_pages {
            if self.is_free_page(page_num)? {
                continue;
            }

            let page_data = self.read_page(page_num)?;
            let header = PageHeader::unpack(&page_data)?;
            let data_start = PageHeader::SIZE;
            let data_end = data_start + header.data_len as usize;
            let data = &page_data[data_start..data_end];

            let mut pos = 0;
            for _ in 0..header.kv_count {
                if pos >= data.len() {
                    break;
                }

                let klen = data[pos] as usize;
                pos += 1;

                if pos + klen > data.len() {
                    break;
                }
                let key = &data[pos..pos + klen];
                pos += klen;

                if pos + 2 > data.len() {
                    break;
                }
                let vlen = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
                pos += 2 + vlen;

                self.key_to_page.insert(key.to_vec(), page_num);
            }
        }

        if self.header.total_pages > 0 {
            self.last_used_page = self.header.total_pages;
        }

        Ok(())
    }


    fn is_free_page(&mut self, page_num: u32) -> Result<bool> {
        let mut current = self.header.free_page_head;
        while current != 0 {
            if current == page_num {
                return Ok(true);
            }
            let page_data = self.read_page(current)?;
            let header = PageHeader::unpack(&page_data)?;
            current = header.next_page;
        }
        Ok(false)
    }


    fn read_page(&mut self, page_num: u32) -> Result<Vec<u8>> {
        if let Some(cached) = self.page_cache.get(page_num) {
            return Ok(cached);
        }

        if page_num < 1 || page_num > self.header.total_pages {
            bail!("Page number out of range");
        }

        let offset = HEADER_SIZE + (page_num - 1) as usize * PAGE_SIZE;
        if offset + PAGE_SIZE > self.mmap.len() {
            bail!("Page offset out of range");
        }

        let page_data = self.mmap[offset..offset + PAGE_SIZE].to_vec();
        self.page_cache.put(page_num, page_data.clone());
        self.last_used_page = page_num;

        Ok(page_data)
    }


    fn write_page(&mut self, page_num: u32, data: &[u8]) -> Result<()> {
        if page_num < 1 || page_num > self.header.total_pages {
            bail!("Page number out of range");
        }

        let offset = HEADER_SIZE + (page_num - 1) as usize * PAGE_SIZE;
        if offset + PAGE_SIZE > self.mmap.len() {
            // 需要扩展文件
            let new_size = (offset + PAGE_SIZE) as u64;
            self.file.set_len(new_size)?;
            self.mmap = unsafe { MmapOptions::new().map_mut(&self.file)? };
        }

        self.mmap[offset..offset + PAGE_SIZE].copy_from_slice(data);
        self.dirty_pages.insert(page_num);
        self.page_cache.put(page_num, data.to_vec());

        Ok(())
    }


    fn flush_pages(&mut self) -> Result<()> {
        if self.dirty_pages.is_empty() {
            return Ok(());
        }

        let max_page = *self.dirty_pages.iter().max().unwrap();
        let required_size = HEADER_SIZE + (max_page as usize) * PAGE_SIZE;
        if required_size > self.mmap.len() {
            self.file.set_len(required_size as u64)?;
            self.mmap = unsafe { MmapOptions::new().map_mut(&self.file)? };
        }

        self.mmap.flush()?;
        self.dirty_pages.clear();
        Ok(())
    }


    fn update_header(&mut self) -> Result<()> {
        self.header.modify_time = current_time_millis();
        let header_data = self.header.pack();
        self.mmap[..DBHeader::SIZE].copy_from_slice(&header_data);
        self.dirty_pages.insert(0); // 标记头为脏页
        self.flush_pages()
    }


    fn allocate_page(&mut self) -> Result<u32> {
        if self.header.free_page_head != 0 {
            let page_num = self.header.free_page_head;
            let page_data = self.read_page(page_num)?;
            let mut header = PageHeader::unpack(&page_data)?;

            self.header.free_page_head = header.next_page;

            header.flags = 0;
            header.kv_count = 0;
            header.data_len = 0;
            header.next_page = 0;

            let mut new_page_data = header.pack().to_vec();
            new_page_data.resize(PAGE_SIZE, 0);
            self.write_page(page_num, &new_page_data)?;

            return Ok(page_num);
        }

        self.header.total_pages += 1;
        let page_num = self.header.total_pages;

        let header = PageHeader {
            flags: 0,
            kv_count: 0,
            data_len: 0,
            next_page: 0,
        };

        let mut page_data = header.pack().to_vec();
        page_data.resize(PAGE_SIZE, 0);

        self.write_page(page_num, &page_data)?;
        Ok(page_num)
    }


    fn free_page(&mut self, page_num: u32) -> Result<()> {
        let header = PageHeader {
            flags: 0x04,
            kv_count: 0,
            data_len: 0,
            next_page: self.header.free_page_head,
        };

        let mut page_data = header.pack().to_vec();
        page_data.resize(PAGE_SIZE, 0);
        self.write_page(page_num, &page_data)?;

        self.header.free_page_head = page_num;
        Ok(())
    }


    fn read_overflow(&mut self, start_page: u32) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut current = start_page;

        while current != 0 {
            let page_data = self.read_page(current)?;
            let header = PageHeader::unpack(&page_data)?;
            let chunk = &page_data[PageHeader::SIZE..PageHeader::SIZE + header.data_len as usize];
            data.extend_from_slice(chunk);
            current = header.next_page;
        }

        Ok(data)
    }


    fn write_overflow(&mut self, data: &[u8]) -> Result<u32> {
        let mut start_page = 0;
        let mut current_page = 0;
        let mut data = data;

        while !data.is_empty() {
            let page_num = self.allocate_page()?;
            if start_page == 0 {
                start_page = page_num;
            }

            let chunk_size = std::cmp::min(PAGE_SIZE - PageHeader::SIZE, data.len());
            let chunk = &data[..chunk_size];
            data = &data[chunk_size..];

            let next_page = if data.is_empty() { 0 } else { self.allocate_page()? };

            let header = PageHeader {
                flags: 0x02,
                kv_count: 0,
                data_len: chunk_size as u16,
                next_page,
            };

            let mut page_data = header.pack().to_vec();
            page_data.extend_from_slice(chunk);
            page_data.resize(PAGE_SIZE, 0);

            self.write_page(page_num, &page_data)?;

            // 更新前一页的next_page指针
            if current_page != 0 {
                let prev_page_data = self.read_page(current_page)?;
                let mut prev_header = PageHeader::unpack(&prev_page_data)?;
                prev_header.next_page = page_num;

                let mut new_prev_data = prev_header.pack().to_vec();
                new_prev_data.extend_from_slice(
                    &prev_page_data[PageHeader::SIZE..PageHeader::SIZE + prev_header.data_len as usize],
                );
                new_prev_data.resize(PAGE_SIZE, 0);
                self.write_page(current_page, &new_prev_data)?;
            }

            current_page = page_num;
        }

        Ok(start_page)
    }


    fn insert_to_page(
        &mut self,
        page_num: u32,
        key: &[u8],
        value: &[u8],
        update_index: bool,
    ) -> Result<()> {
        let page_data = self.read_page(page_num)?;
        let mut header = PageHeader::unpack(&page_data)?;
        let data_start = PageHeader::SIZE;
        let data_end = data_start + header.data_len as usize;
        let data = &page_data[data_start..data_end];

        // 处理溢出
        let overflow_start = if value.len() > OVERFLOW_THRESHOLD {
            let start = self.write_overflow(&value[OVERFLOW_THRESHOLD..])?;
            header.flags |= 0x02;
            start
        } else {
            header.flags &= !0x02;
            0
        };
        header.next_page = overflow_start;

        let value_to_store = if value.len() > OVERFLOW_THRESHOLD {
            &value[..OVERFLOW_THRESHOLD]
        } else {
            value
        };

        // 创建KV条目
        let mut kv_data = Vec::new();
        kv_data.push(key.len() as u8);
        kv_data.extend_from_slice(key);
        kv_data.write_u16::<LittleEndian>(value_to_store.len() as u16)?;
        kv_data.extend_from_slice(value_to_store);

        // 创建新数据
        let mut new_data = data.to_vec();
        new_data.extend_from_slice(&kv_data);

        header.kv_count += 1;
        header.data_len = new_data.len() as u16;

        // 更新页数据
        let mut new_page_data = header.pack().to_vec();
        new_page_data.extend_from_slice(&new_data);
        new_page_data.resize(PAGE_SIZE, 0);

        self.write_page(page_num, &new_page_data)?;

        if update_index {
            self.key_to_page.insert(key.to_vec(), page_num);
        }
        self.last_used_page = page_num;

        Ok(())
    }


    fn update_existing(&mut self, key: &[u8], new_value: &[u8]) -> Result<()> {
        let page_num = *self
            .key_to_page
            .get(key)
            .ok_or_else(|| anyhow!("Key not found"))?;

        let page_data = self.read_page(page_num)?;
        let mut header = PageHeader::unpack(&page_data)?;
        let data_start = PageHeader::SIZE;
        let data_end = data_start + header.data_len as usize;
        let data = &page_data[data_start..data_end];

        let mut pos = 0;
        let mut found = false;
        let mut value_start = 0;
        let mut value_len = 0;
        for _ in 0..header.kv_count {
            let klen = data[pos] as usize;
            pos += 1;

            if pos + klen > data.len() {
                break;
            }
            let current_key = &data[pos..pos + klen];
            pos += klen;

            if pos + 2 > data.len() {
                break;
            }
            value_len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;

            if current_key == key {
                value_start = pos;
                found = true;
                break;
            }

            pos += value_len;
        }

        if !found {
            bail!(KvError::KeyNotFound(String::from_utf8_lossy(key).to_string()));
        }

        // 释放现有溢出页（如果有）
        if header.flags & 0x02 != 0 {
            self.free_overflow(header.next_page)?;
            header.flags &= !0x02;
        }

        // 处理新溢出
        let overflow_start = if new_value.len() > OVERFLOW_THRESHOLD {
            let start = self.write_overflow(&new_value[OVERFLOW_THRESHOLD..])?;
            header.flags |= 0x02;
            start
        } else {
            0
        };
        header.next_page = overflow_start;

        let value_to_store = if new_value.len() > OVERFLOW_THRESHOLD {
            &new_value[..OVERFLOW_THRESHOLD]
        } else {
            new_value
        };

        // 创建新值数据
        let mut new_value_data = Vec::new();
        new_value_data.write_u16::<LittleEndian>(value_to_store.len() as u16)?;
        new_value_data.extend_from_slice(value_to_store);

        // 替换旧值
        let before = &data[..value_start - 2];
        let after = &data[value_start + value_len..];
        let new_data = [before, &new_value_data, after].concat();

        header.data_len = new_data.len() as u16;

        // 更新页数据
        let mut new_page_data = header.pack().to_vec();
        new_page_data.extend_from_slice(&new_data);
        new_page_data.resize(PAGE_SIZE, 0);

        self.write_page(page_num, &new_page_data)?;
        Ok(())
    }


    fn free_overflow(&mut self, start_page: u32) -> Result<()> {
        let mut current = start_page;
        while current != 0 {
            let page_data = self.read_page(current)?;
            let header = PageHeader::unpack(&page_data)?;
            let next_page = header.next_page;
            self.free_page(current)?;
            current = next_page;
        }
        Ok(())
    }


    pub fn close(mut self) -> Result<()> {
        self.flush_pages()?;
        self.update_header()?;
        self.mmap.flush()?;
        self.file.sync_all()?;
        Ok(())
    }
}


fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
