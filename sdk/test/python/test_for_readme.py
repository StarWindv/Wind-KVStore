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