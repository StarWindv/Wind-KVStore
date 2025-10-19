# Wind-KVStore Python SDK  

This SDK provides a Python client for interacting with Wind-KVStore servers.

**Note that the link library has been migrated to the [new project](https://github.com/starwindv/wind-kvstore-lib). You can compile it as needed**

## Installation

```bash
pip install git+https://github.com/StarWindv/Wind-KVStore.git@main#subdirectory=sdk/python
```

## Quick Start

```python
from wind_kvstore.client import WindKVStore

# Connect to local server and open database
with WindKVStore(kv_path="mydata.kv") as kv:
    # Store data
    kv.put_kv("username", "alice")

    # Retrieve data
    print(kv._get_value("username"))  # Outputs: alice

    # Execute batch commands
    result = kv.execute('PUT "email": "alice@example.com"; GET WHERE KEY="email"')
    print(result)
```

## Core Features

### Database Management
| Method               | Description                |
|----------------------|----------------------------|
| `open(path)`         | Opens a database file      |
| `close()`            | Closes current database    |
| `get_current_path()` | Gets current database path |
| `change_kv(path)`    | Switches to a new database |

### Key-Value Operations
| Method               | Description              |
|----------------------|--------------------------|
| `put_kv(key, value)` | Stores a key-value pair  |
| `get_value(key)`     | Retrieves value by key   |
| `del_kv(key)`        | Deletes a key-value pair |

### Advanced Features
| Method                   | Description                          |
|--------------------------|--------------------------------------|
| `compact()`              | Compacts the database                |
| `get_kv_id()`            | Gets database identifier             |
| `set_kv_id(identifier)`  | Sets database identifier             |
| `execute(command)`       | Executes raw commands                |
| `state_execute(command)` | Executes commands and returns status |

## Complete Example

```python
from wind_kvstore.client import WindKVStore

# Create connection (automatically closes using context manager)
with WindKVStore(host="127.0.0.1", port=14514, kv_path="userdata.kv") as kv:
    # Store user information
    kv.put_kv("user:1001", '{"name": "Alice", "email": "alice@example.com"}')
    kv.put_kv("user:1002", '{"name": "Bob", "email": "bob@example.com"}')

    # Retrieve user data
    user_data = kv._get_value("user:1001")
    print(f"User 1001: {user_data}")

    # Batch operations
    commands = '''
        PUT "config:theme": "dark";
        PUT "config:language": "en";
        DEL "temp:session";
    '''
    results = kv.state_execute(commands)
    print(f"Batch operation results: {results}")

    # Set database identifier
    kv.set_kv_id("user_database")
    print(f"Database ID: {kv.get_kv_id()}")

    # Database maintenance
    kv.compact()
```

## Configuration Options

Initialization parameters:
```python
WindKVStore(
    host="127.0.0.1",    # Server address
    port=14514,          # Server port
    kv_path="",          # Database file path
    use_https=False,     # Enable HTTPS
    check_active=True,   # Automatically check session activation status
    use_time_to_check_active=False  # Use timestamps for session checking
)
```

## Command Execution Format

The `execute()` method supports batch operations:
```python
# Multi-command format (semicolon separated)
commands = '''
    PUT "key1": "value1";
    PUT "key2": "value2";
    GET "key1";
    IDENTIFIER SET "new_id";
'''

# Returns structured results
result = kv.execute(commands)
```

## Additional Resources

- [Wind-KVStore Server Documentation](https://github.com/StarWindv/Wind-KVStore/blob/main/doc/readme_server.md)
- [Test Code](https://github.com/StarWindv/Wind-KVStore/tree/main/sdk/test/python/test.py)
