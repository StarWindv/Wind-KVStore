## 💻 交互式 Shell 使用指南

### 启动界面
```
Welcome to Wind-KVStore!

██╗    ██╗    ██╗    ███╗   ██╗    ██████╗ 
██║    ██║    ██║    ████╗  ██║    ██╔══██╗
██║ █╗ ██║    ██║    ██╔██╗ ██║    ██║  ██║
██║███╗██║    ██║    ██║╚██╗██║    ██║  ██║
╚███╔███╔╝    ██║    ██║ ╚████║    ██████╔╝
 ╚══╝╚══╝     ╚═╝    ╚═╝  ╚═══╝    ╚═════╝ 

vVersion [compiled YYYY-MM-DD hh:mm:ss]
Type ".help;" for usage hints.

KVStore > 
```

### Shell操作示例

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
注意：当前不支持`--`等注释格式
### 📚 Shell 命令参考

| 命令                    | 描述       | 示例                        |
|-----------------------|----------|---------------------------|
| `.open <path>`        | 打开/创建数据库 | `.open data.db;`          |
| `.close`              | 关闭当前数据库  | `.close;`                 |
| `.help`               | 查看帮助信息   | `.help;`                  |
| `.clear`              | 清屏       | `.clear;`                 |
| `.title`              | 显示标题信息   | `.title;`                 |
| `.quit`               | 退出程序     | `.quit;`                  |
| `PUT "key":"value"`   | 插入键值对    | `PUT "name":"Alice";`     |
| `GET WHERE KEY="key"` | 查询键值     | `GET WHERE KEY="age";`    |
| `DEL WHERE KEY="key"` | 删除键值     | `DEL WHERE KEY="temp";`   |
| `COMPACT`             | 压缩数据库    | `COMPACT;`                |
| `IDENTIFIER SET "id"` | 设置数据库标识符 | `IDENTIFIER SET "AppDB";` |
| `IDENTIFIER GET`      | 获取数据库标识符 | `IDENTIFIER GET;`         |
