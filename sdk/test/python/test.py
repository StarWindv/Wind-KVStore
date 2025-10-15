# """
# If you want to run this test multiple times,
# and need the `pass` result,
# you should delete the existing kv and wal files used.
# """


# from wind_kvstore.pool import WindKVStore
# from wind_kvstore.client import WindKVStore
from wind_kvstore.auto import auto
import time

WindKVStore = auto("pool")

class TestWindKVStore:
    def test_basic_operations(self):
        """Test basic key value pair operations: open, insert, query, delete, and close databases"""
        print(f"\n>> Start {self.test_basic_operations.__doc__}")
        kv = WindKVStore()

        # 测试打开数据库
        print("[INFO] Open kv.")
        assert kv.open("test.kv"), "[Err ] Failed when open kv."
        print(f"[INFO] Current session-id: {kv.session_id}")
        print(f"[INFO] Current kv path: {kv.get_current_path()}")

        # 测试插入数据
        print("\n[INFO] Insert key-value pairs")
        test_data = [("name", "Alice"), ("age", "25"), ("city", "New York")]
        for key, value in test_data:
            assert kv.put_kv(key, value), f"[Err ] Failed when insert {key}:{value}"

        # 测试查询数据
        print("\n[INFO] Query key-value pairs")
        for key, expected_value in test_data:
            actual_value = kv.get_value(key)
            print(f"{key}: {actual_value}")
            assert actual_value == expected_value, \
                f"[Err] Failed when query {key}, expect: {expected_value}"

        # 测试删除数据
        print("\n[INFO] Delete key-value pairs")
        delete_key = "city"
        assert kv.del_kv(delete_key), f"[Err ] Failed when delete {delete_key}"
        print(f"[INFO] Already deleted {delete_key}")

        # 验证删除结果
        deleted_value = kv.get_value(delete_key)
        print(f"[INFO] Attempt to query a deleted key:"
              f"`{delete_key}: {deleted_value}`")
        assert deleted_value is None or deleted_value == "", f"[Err ] Failed when delete `{delete_key}`"

        # 测试关闭数据库
        print("\n[INFO] Try to close current kv.")
        assert kv.close(), "[Err ] Failed to close current kv."
        print("[INFO] Database already closed.")

        print(f"<< End {self.test_basic_operations.__doc__}")

    def test_advanced_operations(self):
        """Testing advanced operations: identifier setting, compact operation, and batch command execution"""
        print(f"\n>> Start {self.test_advanced_operations.__doc__}")
        with WindKVStore() as kv:
            print("[INFO] Open advanced.kv")
            kv.open("advanced.kv")

            # 测试设置和获取标识符
            print("\n[INFO] Test identifier operations")
            test_identifier = "test_db_123"
            kv.set_kv_id(test_identifier)
            current_identifier = kv.get_kv_id()
            print(f"[INFO] Current identifier: {current_identifier}")
            assert current_identifier == test_identifier, "[Err ] Set identifier failed"

            # 测试compact操作
            print("\n[INFO] Test compact operation")
            assert kv.compact(), "[Err ] Compact operation failed"
            print("[INFO] Database compacted successfully")

            # 测试execute命令
            print("\n[INFO] Test execute command")
            commands = """
            PUT "color":"blue";\
            PUT "size":"large";\
            IDENTIFIER SET "advanced_db_456";
            """
            result = kv.execute(commands)
            print("[INFO] Execution result:")
            print(result)

            # 验证执行结果
            print("\n[INFO] Verify execution results")
            assert kv.get_value("color") == "blue", "[Err ] Incorrect color value after execution"
            assert kv.get_value("size") == "large", "[Err ] Incorrect size value after execution"
            assert kv.get_kv_id() == "advanced_db_456", "[Err ] Identifier not updated after execution"

            print(f"[INFO] color: {kv.get_value('color')}")
            print(f"[INFO] size: {kv.get_value('size')}")
            print(f"[INFO] New identifier: {kv.get_kv_id()}")

            # 测试state_execute命令
            print("\n[INFO] Test state_execute command")
            commands = """
                PUT "colors":"white";\
                PUT "sizes":"small";\
                IDENTIFIER SET "l l l l";
                """
            result = kv.state_execute(commands)
            print("[INFO] Execution result:")
            print(result)

            # 验证执行结果
            print("\n[INFO] Verify execution results")
            assert kv.get_value("colors") == "white", "[Err ] Incorrect colors value after execution"
            assert kv.get_value("sizes") == "small", "[Err ] Incorrect sizes value after execution"
            assert kv.get_kv_id() == "l l l l", "[Err ] Identifier not updated after execution"

            print(f"[INFO] colors: {kv.get_value('colors')}")
            print(f"[INFO] sizes: {kv.get_value('sizes')}")
            print(f"[INFO] New identifier: {kv.get_kv_id()}")

        print(f"<< End {self.test_advanced_operations.__doc__}")

    def test_session_management(self):
        """Test session management function: session activity status check and automatic reconnection mechanism"""
        print(f"\n>> Start {self.test_session_management.__doc__}")
        with WindKVStore(use_time_to_check_active=True) as kv:
            print("[INFO] Open session_test.kv")
            kv.open("session_test.kv")

            print("[INFO] Initial session status:")
            print(f"[INFO] Active: {kv.is_activate()}")
            print(f"[INFO] Session start: {time.ctime(kv.session_start)}")
            assert kv.is_activate(), "[Err ] Initial session should be active"

            # 模拟长时间不操作（超过30分钟）
            print("\n[INFO] Simulate 30 minutes of inactivity")
            kv.session_start = time.time() + 1801

            # 尝试操作，应该自动重新连接
            print("[INFO] Try operation after inactivity")
            assert kv.put_kv("test_key", "test_value"), "[Err ] Auto-reconnect failed"
            print("[INFO] Operation succeeded, auto-reconnect verified")

            # 验证会话状态恢复活跃
            assert kv.is_activate(), "[Err ] Session should be active after operation"

        print(f"<< End {self.test_session_management.__doc__}")

    def test_error_handling(self):
        """Test error handling mechanism: Verify the correct handling of abnormal situations"""
        print(f"\n>> Start {self.test_error_handling.__doc__}")
        kv = WindKVStore()

        # 测试未打开数据库时的操作
        print("[INFO] Test operations without opening DB")
        try:
            kv.get_current_path()
            assert False, "[Err ] Should raise exception when DB not open"
        except Exception as e:
            print(f"[INFO] Expected error: {str(e)}")

        # 测试空键值操作
        print("\n[INFO] Test empty key/value operations")
        print("[INFO] Open error_test.kv")
        kv.open("error_test.kv")

        try:
            kv.put_kv("", "value")
            assert False, "[Err ] Should raise exception for empty key"
        except ValueError as e:
            print(f"[INFO] Expected error: {str(e)}")

        try:
            kv.put_kv("key", "")
            assert False, "[Err ] Should raise exception for empty value"
        except ValueError as e:
            print(f"[INFO] Expected error: {str(e)}")

        print("[INFO] Close error_test.kv")
        kv.close()

        print(f"<< End {self.test_error_handling.__doc__}")

    def run_all_tests(self):
        """运行所有测试用例"""
        print(">> Start WindKVStore tests")

        # 运行各个测试方法
        self.test_basic_operations()
        self.test_advanced_operations()
        self.test_session_management()
        self.test_error_handling()

        print("\n<< End All tests completed")

if __name__ == "__main__":
    # 创建测试实例并运行所有测试
    test = TestWindKVStore()
    test.run_all_tests()
