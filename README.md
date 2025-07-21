# Wind-KVStore

[中文](./README_CN.md)

![GitHub License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust Version](https://img.shields.io/badge/rust-1.65%2B-orange)

Wind-KVStore is a lightweight, efficient, and persistent key-value storage engine implemented in Rust. This project combines a high-performance storage engine with a user-friendly command-line interface, making it suitable for scenarios requiring local persistent storage.

## ✨ Features

- **📁 Persistent Storage** - Data is safely written to disk with crash recovery support
- **📝 Write-Ahead Log (WAL)** - Ensures operation atomicity and durability
- **⚡ Memory-Mapped Files** - Provides efficient file access performance
- **🗂️ LRU Cache** - Automatically manages hot data caching
- **🔢 Paged Storage** - Supports overflow pages for handling large values
- **♻️ Free Page Management** - Efficiently reuses disk space
- **🗜️ Database Compression** - Optimizes storage space utilization
- **💻 Interactive Shell** - Provides an intuitive command-line interface

## 🚀 Quick Start

### Prerequisites

- Rust toolchain ([installation guide](https://www.rust-lang.org/tools/install))

### Installation and Running

```bash
# Clone the repository
git clone https://github.com/starwindv/wind-kvstore
cd wind-kvstore

# Build the project
cargo build --release

# Run the interactive shell
cargo run
```

## 💻 Interactive Shell Guide

### Startup Interface
```
Welcome to Wind-KVStore!

██╗    ██╗    ██╗    ███╗   ██╗    ██████╗ 
██║    ██║    ██║    ████╗  ██║    ██╔══██╗
██║ █╗ ██║    ██║    ██╔██╗ ██║    ██║  ██║
██║███╗██║    ██║    ██║╚██╗██║    ██║    ██║
╚███╔███╔╝    ██║    ██║ ╚████║    ██████╔╝
 ╚══╝╚══╝     ╚═╝    ╚═╝  ╚═══╝    ╚═════╝ 

v0.0.1 [compiled 2023-10-15 14:30:00]
Type ".help;" for usage hints.

KVStore > 
```

### Shell Operation Examples

```
.open my_database.db;

PUT "username":"john_doe", "email":"john@example.com";

GET WHERE KEY="username";

DEL WHERE KEY="email";

COMPACT;

IDENTIFIER SET "UserDatabase";

IDENTIFIER GET;

.close;

.quit;
```
Note: Comment formats like `--` are not currently supported.

### 📚 Shell Command Reference

| Command | Description | Example |
|---------|-------------|---------|
| `.open <path>` | Open/create a database | `.open data.db;` |
| `.close` | Close the current database | `.close;` |
| `.help` | View help information | `.help;` |
| `.clear` | Clear the screen | `.clear;` |
| `.title` | Display title information | `.title;` |
| `.quit` | Exit the program | `.quit;` |
| `PUT "key":"value"` | Insert a key-value pair | `PUT "name":"Alice";` |
| `GET WHERE KEY="key"` | Retrieve a value by key | `GET WHERE KEY="age";` |
| `DEL WHERE KEY="key"` | Delete a key-value pair | `DEL WHERE KEY="temp";` |
| `COMPACT` | Compact the database | `COMPACT;` |
| `IDENTIFIER SET "id"` | Set the database identifier | `IDENTIFIER SET "AppDB";` |
| `IDENTIFIER GET` | Get the database identifier | `IDENTIFIER GET;` |

## 📦 Using as a Library

### Adding Dependency

Add to `Cargo.toml`:

```toml
[dependencies]
wind-kvstore = { git = "https://github.com/starwindv/wind-kvstore" }
```

### Library Usage Example

```rust
use wind_kvstore::KVStore;
use anyhow::Result;

fn main() -> Result<()> {
    // Open the database
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
    
    // Compact the database
    store.compact()?;
    
    // Close the database
    store.close()?;
    
    Ok(())
}
```

### Core API

```rust
impl KVStore {
    /// Open or create a database
    pub fn open<P: AsRef<Path>>(
        path: P, 
        db_identifier: Option<&str>
    ) -> Result<Self>;
    
    /// Store a key-value pair
    pub fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()>;
    
    /// Retrieve a value by key
    pub fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    
    /// Delete a key-value pair
    pub fn delete(&mut self, key: &[u8]) -> Result<()>;
    
    /// Compact the database
    pub fn compact(&mut self) -> Result<()>;
    
    /// Get the database identifier
    pub fn get_identifier(&self) -> &str;
    
    /// Set the database identifier
    pub fn set_identifier(&mut self, identifier: &str) -> Result<()>;
    
    /// Close the database
    pub fn close(mut self) -> Result<()>;
}
```

## 🏗️ Project Structure

```
.
├── build.rs            # Build script (records compilation time)
├── Cargo.toml          # Project configuration and dependency management
├── README.md           # Project documentation
├── README_CN.md        # Chinese project documentation
└── src
    ├── kvstore.rs      # Core implementation of the key-value storage engine
    ├── main.rs         # Program entry point
    └── shell.rs        # Implementation of the interactive command-line shell
```

## ⚙️ Technical Implementation

### Storage Architecture
```
+-----------------------+
|      File Header (128B) |
+-----------------------+
|      Page Header (16B)  |
+-----------------------+
|       Key-Value Data    |
+-----------------------+
|      Overflow Page Pointer |
+-----------------------+
|         ...           |
+-----------------------+
```

### Key Features

1. **Paged Storage**  
   - Fixed-size pages (1KB by default)
   - Supports overflow pages for large values
   - Free page linked list management

2. **Write-Ahead Log**  
   - Operation log recording
   - Automatic recovery after crashes
   - Atomic operation guarantees

3. **Cache Management**  
   - LRU caching strategy
   - Automatic hot data management
   - 100KB default cache size

4. **Space Optimization**  
   - Automatic free page recycling
   - Online database compression
   - Efficient storage layout

## 🤝 Contribution Guide

We welcome contributions in any form!
We recommend that when making contributions, you:
 - Update relevant documentation
 - Submit clear PR descriptions

## 📜 License

This project is licensed under the [MIT License](https://github.com/StarWindv/Wind-KVStore/LICENSE)
