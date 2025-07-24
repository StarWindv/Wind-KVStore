## 💻 Interactive Shell User Guide

### Startup Interface
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
Note: Comments using `--` format are currently not supported

### 📚 Shell Command Reference

| Command                 | Description        | Example                     |
|-------------------------|--------------------|-----------------------------|
| `.open <path>`         | Open/create database | `.open data.db;`           |
| `.close`               | Close current database | `.close;`                |
| `.help`                | Show help information | `.help;`                 |
| `.clear`               | Clear screen        | `.clear;`                |
| `.title`               | Display title info  | `.title;`                |
| `.quit`                | Exit program        | `.quit;`                 |
| `PUT "key":"value"`    | Insert key-value pair | `PUT "name":"Alice";`    |
| `GET WHERE KEY="key"`  | Query value by key  | `GET WHERE KEY="age";`   |
| `DEL WHERE KEY="key"`  | Delete key-value    | `DEL WHERE KEY="temp";`  |
| `COMPACT`              | Compact database    | `COMPACT;`               |
| `IDENTIFIER SET "id"`  | Set database identifier | `IDENTIFIER SET "AppDB";` |
| `IDENTIFIER GET`       | Get database identifier | `IDENTIFIER GET;`        |

### Usage Notes
1. All commands must end with a semicolon (;)
2. Keys and values must be enclosed in double quotes
3. Multiple PUT operations can be combined in one command separated by commas
4. The shell maintains session state until explicitly closed or quit

### Special Features
- Command history (up/down arrows)
- Tab auto-completion for commands
- Syntax highlighting for better readability
- Session persistence across commands
