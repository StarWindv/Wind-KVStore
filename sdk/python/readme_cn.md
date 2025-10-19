# Wind-KVStore Python SDK

此SDK提供了与Wind-KVStore服务器交互的Python客户端

**注意, 链接库已迁移到[新项目](https://github.com/starwindv/wind-kvstore-lib)，您可以根据需要来自行编译**

支持键值存储操作、数据库管理和批量命令执行。

## 安装

```bash
pip install git+https://github.com/StarWindv/Wind-KVStore.git@main#subdirectory=sdk/python
```

## 快速开始

```python
from wind_kvstore.client import WindKVStore

# 连接到本地服务器并打开数据库
with WindKVStore(kv_path="mydata.kv") as kv:
    # 存储数据
    kv.put_kv("username", "alice")

    # 检索数据
    print(kv._get_value("username"))  # 输出: alice

    # 执行批量命令
    result = kv.execute('PUT "email": "alice@example.com"; GET WHERE KEY="email"')
    print(result)
```

## 核心功能

### 数据库管理
| 方法                   | 描述        |
|----------------------|-----------|
| `open(path)`         | 打开数据库文件   |
| `close()`            | 关闭当前数据库   |
| `get_current_path()` | 获取当前数据库路径 |
| `change_kv(path)`    | 切换到新数据库   |

### 键值操作
| 方法                   | 描述      |
|----------------------|---------|
| `put_kv(key, value)` | 存储键值对   |
| `get_value(key)`     | 获取键对应的值 |
| `del_kv(key)`        | 删除键值对   |

### 高级功能
| 方法                       | 描述        |
|--------------------------|-----------|
| `compact()`              | 压缩数据库     |
| `get_kv_id()`            | 获取数据库标识符  |
| `set_kv_id(identifier)`  | 设置数据库标识符  |
| `execute(command)`       | 执行原始命令    |
| `state_execute(command)` | 执行命令并返回状态 |

## 完整示例

```python
from wind_kvstore.client import WindKVStore

# 创建连接（使用上下文管理器自动关闭）
with WindKVStore(host="127.0.0.1", port=14514, kv_path="userdata.kv") as kv:
    # 存储用户信息
    kv.put_kv("user:1001", '{"name": "Alice", "email": "alice@example.com"}')
    kv.put_kv("user:1002", '{"name": "Bob", "email": "bob@example.com"}')

    # 获取用户数据
    user_data = kv._get_value("user:1001")
    print(f"User 1001: {user_data}")

    # 批量操作
    commands = '''
        PUT "config:theme": "dark";
        PUT "config:language": "en";
        DEL "temp:session";
    '''
    results = kv.state_execute(commands)
    print(f"Batch operation results: {results}")

    # 设置数据库标识符
    kv.set_kv_id("user_database")
    print(f"Database ID: {kv.get_kv_id()}")

    # 数据库维护
    kv.compact()
```

## 配置选项

初始化参数：
```python
WindKVStore(
    host="127.0.0.1",    # 服务器地址
    port=14514,           # 服务器端口
    kv_path="",           # 数据库文件路径
    use_https=False,      # 启用HTTPS
    check_active=True,    # 自动检查会话激活状态
    use_time_to_check_active=False  # 使用时间戳检查会话
)
```

## 命令执行格式

`execute()` 方法支持批量操作：
```python
# 多命令格式（分号分隔）
commands = '''
    PUT "key1": "value1";
    PUT "key2": "value2";
    GET "key1";
    IDENTIFIER SET "new_id";
'''

# 返回结构化结果
result = kv.execute(commands)
```

## 更多资源

- [Wind-KVStore 服务器文档](https://github.com/StarWindv/Wind-KVStore/blob/main/doc/readme_server_cn.md)
- [测试代码](https://github.com/StarWindv/Wind-KVStore/tree/main/sdk/test/python/test.py)
