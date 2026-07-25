# Wind-KVStore

You can find the version under development with new features [here](https://github.com/StarWindv/Wind-KVStore/tree/dev)

[中文](https://github.com/StarWindv/Wind-KVStore/blob/main/README_CN.md)

![GitHub License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust Version](https://img.shields.io/badge/rust-1.85%2B-orange)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/StarWindv/Wind-KVStore)

Wind-KVStore is a lightweight, efficient, and persistent key-value storage engine implemented in Rust. The project combines a high-performance storage engine with a user-friendly command-line interface and net server, making it suitable for scenarios requiring local persistent storage.

## ✨ Features

- **📁 Persistent Storage** - Data securely written to disk with crash recovery support
- **📝 Write-Ahead Log (WAL)** - Ensures operation atomicity and durability
- **⚡ Memory-Mapped Files** - Provides high-performance file access
- **🗂️ LRU Caching** - Automatically manages hot data caching
- **🔢 Paged Storage** - Supports overflow pages for large value data
- **♻️ Free Page Management** - Efficient disk space reuse
- **🗜️ Database Compression** - Optimizes storage space utilization
- **>_ Interactive Shell** - Offers intuitive command-line interface
- **🖥️ Server** - Provides clean server interface with built-in session management

## 🚀 Quick Start

### Prerequisites

- Rust toolchain ([Installation Guide](https://www.rust-lang.org/tools/install))

### Installation & Running

```bash
# Clone repository
git clone https://github.com/starwindv/wind-kvstore
cd wind-kvstore

# Build project
cargo build --release

# Run interactive Shell
cargo run --bin wkshell --release

# Run server
cargo run --bin wkserver --release
```

## \>_ Interactive Shell Guide
[Shell Documentation](https://github.com/StarWindv/Wind-KVStore/blob/main/doc/readme_shell.md)

## 🖥️ Server Guide
[Server Documentation](https://github.com/starwindv/wind-kvstore/blob/main/doc/readme_server.md)

## 📦 Using as a Library

### Add Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
wind-kvstore = { git = "https://github.com/starwindv/wind-kvstore" }
```

### Library Usage Example

```rust
use wind_kvstore::KVStore;
use anyhow::Result;

fn main() -> Result<()> {
    // Open database
    let mut store = KVStore::open("app_data.db", Some("MyAppDB"))?;
    
    // Store data
    store.put(b"username", b"alice")?;
    
    // Retrieve data
    if let Some(value) = store.get(b"username")? {
        println!("Username: {}", String::from_utf8_lossy(&value));
    }
    
    // Delete data
    store.delete(b"username")?;
    
    // Set database identifier
    store.set_identifier("UserDatabase")?;
    
    // Compact database
    store.compact()?;
    
    // Close database
    store.close()?;
    
    Ok(())
}
```

### Core API

```rust
impl KVStore {
    /// Open or create database
    pub fn open<P: AsRef<Path>>(
        path: P, 
        db_identifier: Option<&str>
    ) -> Result<Self>{}
    
    /// Store key-value pair
    pub fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()>{}
    
    /// Retrieve value
    pub fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>>{}
    
    /// Delete key-value
    pub fn delete(&mut self, key: &[u8]) -> Result<()>{}
    
    /// Compact database
    pub fn compact(&mut self) -> Result<()>{}
    
    /// Get database identifier
    pub fn get_identifier(&self) -> &str{}
    
    /// Set database identifier
    pub fn set_identifier(&mut self, identifier: &str) -> Result<()>{}
    
    /// Close database
    pub fn close(mut self) -> Result<()>{}
}
```

## 🏗️ Project Structure

```plaintext
├── build.rs            # Build script (records compilation time)
├── Cargo.toml          # Project configuration and dependency management
├── doc/                # Another documentation(Chinese and English)
├── README.md           # Project documentation (English)
├── README_CN.md        # Project documentation (Chinese)
├── LICENSE             # MIT
├── sdk
│   ├── python/         # Wind-KVStore server's sdk for python
│   └── test/           # Test sdk
├── GUI
│   ├── src/            # Wind KVStore's visualize interface
│   └── LICENSE         # GPLv3
└── src
    ├── config.rs       # Server configuration loader
    ├── kvstore.rs      # Core KV storage engine implementation
    ├── server.rs       # Server main logic
    ├── server_main.rs  # Server entry point
    ├── shell.rs        # Interactive shell main logic
    ├── shell_main.rs   # Interactive shell entry point
    └── utils.rs        # Utility functions
```

## 📦 Another Modules

### Visualize Interface
- [UI](./GUI)

### Python SDK
- [Python SDK](./sdk/python)

### Python Lib
- [Python Lib](https://github.com/starwindv/wind-kvstore-lib)


## ⚙️ Technical Implementation

### Storage Architecture
```
+-----------------------+
|    File Header (128B) |
+-----------------------+
|    Page Header (16B)  |
+-----------------------+
|      Key-Value Data   |
+-----------------------+
|    Overflow Page Ptr  |
+-----------------------+
|         ...           |
+-----------------------+
```

### Key Features

1. **Paged Storage**
    - Fixed-size pages (default 1KB)
    - Overflow page support for large values
    - Free page linked list management

2. **Write-Ahead Log**
    - Operation logging
    - Automatic crash recovery
    - Atomic operation guarantee

3. **Cache Management**
    - LRU caching strategy
    - Automatic hot data management
    - Default 100KB cache size

4. **Space Optimization**
    - Automatic free page recycling
    - Online database compression
    - Efficient storage layout


## 🤝 Contribution Guide

We welcome all forms of contributions!
When contributing, we recommend:
- Updating relevant documentation
- Submitting clear PR descriptions
- Not worrying about communication language - we accept both English and Chinese descriptions. If you're not comfortable with these languages, feel free to use your preferred language

## 📜 License

This project is licensed under the [MIT License](https://github.com/StarWindv/Wind-KVStore/LICENSE)
