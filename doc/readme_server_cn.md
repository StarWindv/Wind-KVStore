# Wind-KVStore 服务器文档

## 目录
1. [快速开始](#快速开始)
2. [API 参考](#api-参考)
    - [数据库操作](#数据库操作)
    - [数据操作](#数据操作)
    - [管理功能](#管理功能)
    - [通用命令接口](#通用命令接口)
3. [多语言示例](#多语言示例)
4. [高级特性](#高级特性)
5. [最佳实践](#最佳实践)

## 快速开始

### 启动服务器
```bash
cargo run --bin server --release
# 默认监听: http://127.0.0.1:14514
```

### 基础流程示例
```cmd
# 1. 创建会话并打开数据库
curl -v -X POST http://127.0.0.1:14514/api/open ^
  -H "Content-Type: application/json" ^
  -d "{\"path\":\"test.db\"}"
# 不要忘记输入端口号 

# 2. 存储数据
curl -X POST http://127.0.0.1:14514/api/put ^
  -H "X-Session-ID: YOUR_SESSION_ID" ^
  -H "Content-Type: application/json" ^
  -d "[{\"key\":\"user1\",\"value\":\"Alice\"},{\"key\":\"user2\",\"value\":\"Bob\"}]"

# 3. 查询数据
curl "http://127.0.0.1:14514/api/get?key=user1" ^
  -H "X-Session-ID: YOUR_SESSION_ID"

# 4. 关闭连接
curl http://127.0.0.1:14514/api/close ^
  -H "X-Session-ID: YOUR_SESSION_ID"
```

## API 参考

### 数据库操作

| 端点             | 方法   | 描述        | 参数示例                  |
|----------------|------|-----------|-----------------------|
| `/api/open`    | POST | 打开数据库     | `{"path": "data.db"}` |
| `/api/close`   | GET  | 关闭当前数据库   | 需 Session ID          |
| `/api/current` | GET  | 获取当前数据库路径 | 需 Session ID          |

### 数据操作

| 端点         | 方法   | 描述   | 参数示例                              |
|------------|------|------|-----------------------------------|
| `/api/put` | POST | 批量写入 | `[{"key":"k1","value":"v1"},...]` |
| `/api/get` | GET  | 单键查询 | `?key=target_key`                 |
| `/api/del` | POST | 删除键值 | `{"key":"target_key"}`            |

### 管理功能

| 端点             | 方法   | 描述      | 
|----------------|------|---------|
| `/api/compact` | GET  | 压缩数据库   |
| `/api/id/get`  | GET  | 获取数据库标识 |
| `/api/id/set`  | POST | 设置数据库标识 |

### 通用命令接口

#### `/api/execute` 端点
- **方法**: POST
- **Content-Type**: `text/plain`
- **必需头**: `X-Session-ID`

**支持命令格式**:
```bash
# 数据写入
PUT "key1":"value1";

# 数据查询
GET WHERE KEY="key1";

# 数据删除
DEL WHERE KEY="key1";

# 数据库维护
COMPACT;
IDENTIFIER SET "new_name";
IDENTIFIER GET;
```

**响应格式**:
```json
{
  "status": "success|error",
  "result": "执行结果或错误信息"
}
```

## 多语言示例

### Python
```python
import requests

# 打开数据库
res = requests.post("http://localhost:14514/api/open", 
                   json={"path": "demo.db"})
session_id = res.headers["X-Session-ID"]

# 执行命令
requests.post("http://localhost:14514/api/execute",
             headers={"X-Session-ID": session_id},
             data='PUT "lang":"python";')
```
[We also provide a simpler and more user-friendly SDK to assist you in using it.](https://github.com/StarWindv/Wind-KVStore/tree/main/sdk/python)

### JavaScript
```javascript
const axios = require('axios');

// 执行复合命令
axios.post('http://localhost:14514/api/execute',
  'PUT "counter":"1";\nGET WHERE KEY="counter";',
  {
    headers: {
      'X-Session-ID': sessionId,
      'Content-Type': 'text/plain'
    }
  }
);
```

### Java
```java
// 执行查询命令
String command = "GET WHERE KEY=\"ip\";";
HttpRequest request = HttpRequest.newBuilder()
    .uri(URI.create("http://localhost:14514/api/execute"))
    .header("X-Session-ID", sessionId)
    .header("Content-Type", "text/plain")
    .POST(HttpRequest.BodyPublishers.ofString(command))
    .build();
```

## 高级特性

### 会话管理
- 自动创建会话（响应头返回`X-Session-ID`）
- 30分钟无操作自动过期
- 后台5分钟清理闲置会话

## 最佳实践

1. **会话管理**：
    - 及时关闭不再使用的会话
    - 长时间操作需定期发送请求保持会话

2. **API选择**：
    - 简单操作使用专用API端点
    - 复杂批量处理使用`/api/execute`

3. **错误处理**：
    - 检查HTTP状态码
    - 解析JSON错误信息

4. **性能建议**：
    - 批量操作使用数组格式
    - 大值数据分片存储

> 提示：所有键值必须用双引号包裹, 键值之间的冒号两侧不能有空格
>
> 在使用`/api/execute`路由时，多条命令之间请使用分号分隔
