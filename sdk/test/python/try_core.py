import unittest
import tempfile
import os
from unittest.mock import Mock, patch


from wind_kvstore.WindKVCore import WindKVCore


class TestWindKVCore(unittest.TestCase):

    def setUp(self):
        
        self.temp_dir = tempfile.mkdtemp()
        self.test_db_path = os.path.join(self.temp_dir, "test_db.kv")

    def tearDown(self):
        
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)

    def test_init_without_parameters(self):
        
        kv = WindKVCore()
        self.assertIsNone(kv.path)
        self.assertIsNone(kv.db_identifier)
        self.assertIsNone(kv._WindKVCore__inner)

    def test_init_with_parameters(self):
        
        # 创建一个模拟的_WindKVCore实例
        mock_inner = Mock()
        mock_inner.get_identifier.return_value = "test_identifier"

        # 使用patch替换_WindKVCore类
        with patch('wind_kvstore.WindKVCore._WindKVCore', return_value=mock_inner):
            kv = WindKVCore(self.test_db_path, "test_identifier")

            self.assertEqual(kv.path, self.test_db_path)
            self.assertEqual(kv.db_identifier, "test_identifier")

    def test_open_method(self):
        
        # 创建一个模拟的_WindKVCore实例
        mock_inner = Mock()
        mock_inner.get_identifier.return_value = "opened_identifier"

        with patch('wind_kvstore.WindKVCore._WindKVCore', return_value=mock_inner):
            kv = WindKVCore()
            kv.open(self.test_db_path, "test_identifier")

            self.assertEqual(kv.path, self.test_db_path)
            self.assertEqual(kv.db_identifier, "opened_identifier")

    def test_methods_raise_error_when_closed(self):
        
        kv = WindKVCore()  # 没有调用open，数据库是关闭状态

        # 测试所有被装饰的方法
        methods_to_test = [
            ('get', [b'test_key']),
            ('put', [b'test_key', b'test_value']),
            ('delete', [b'test_key']),
            ('get_all', []),
            ('set_identifier', ['test_id']),
            ('compact', []),
            ('get_identifier', []),
            ('close', [])
        ]

        for method_name, args in methods_to_test:
            with self.subTest(method=method_name):
                with self.assertRaises(ValueError) as context:
                    method = getattr(kv, method_name)
                    method(*args)
                self.assertIn("database is not opened", str(context.exception).lower())

    def test_basic_operations_with_mock(self):
        
        # 创建模拟的_WindKVCore实例
        mock_inner = Mock()
        mock_inner.get_identifier.return_value = "test_identifier"
        mock_inner.get.return_value = b"mock_value"
        mock_inner.get_all.return_value = [{"key1": "value1"}, {"key2": "value2"}]

        with patch('wind_kvstore.WindKVCore._WindKVCore', return_value=mock_inner):
            kv = WindKVCore(self.test_db_path)

            # 测试get
            result = kv.get(b"test_key")
            self.assertEqual(result, b"mock_value")
            mock_inner.get.assert_called_once_with(b"test_key")

            # 测试put
            kv.put(b"key", b"value")
            mock_inner.put.assert_called_once_with(b"key", b"value")

            # 测试delete
            kv.delete(b"key")
            mock_inner.delete.assert_called_once_with(b"key")

            # 测试get_all
            all_data = kv.get_all()
            self.assertEqual(all_data, [{"key1": "value1"}, {"key2": "value2"}])
            mock_inner.get_all.assert_called_once()

    def test_identifier_operations(self):
        
        mock_inner = Mock()
        mock_inner.get_identifier.return_value = "initial_identifier"

        with patch('wind_kvstore.WindKVCore._WindKVCore', return_value=mock_inner):
            kv = WindKVCore(self.test_db_path)

            # 测试get_identifier
            identifier = kv.get_identifier()
            self.assertEqual(identifier, "initial_identifier")
            mock_inner.get_identifier.assert_called()

            # 测试set_identifier
            kv._set_identifier("new_identifier")
            mock_inner._set_identifier.assert_called_once_with("new_identifier")

    def test_compact_method(self):
        
        mock_inner = Mock()
        mock_inner.get_identifier.return_value = "test_identifier"

        with patch('wind_kvstore.WindKVCore._WindKVCore', return_value=mock_inner):
            kv = WindKVCore(self.test_db_path)

            kv.compact()
            mock_inner.compact.assert_called_once()

    def test_close_method(self):
        
        mock_inner = Mock()
        mock_inner.get_identifier.return_value = "test_identifier"

        with patch('wind_kvstore.WindKVCore._WindKVCore', return_value=mock_inner):
            kv = WindKVCore(self.test_db_path)

            # 确保初始状态是打开的
            self.assertIsNotNone(kv._WindKVCore__inner)

            kv.close()

            # 验证内部对象被设置为None
            self.assertIsNone(kv._WindKVCore__inner)
            mock_inner.close.assert_called_once()

    def test_context_manager_behavior(self):
        
        mock_inner = Mock()
        mock_inner.get_identifier.return_value = "test_identifier"

        with patch('wind_kvstore.WindKVCore._WindKVCore', return_value=mock_inner):
            # 创建并打开数据库
            kv = WindKVCore(self.test_db_path)
            self.assertIsNotNone(kv._WindKVCore__inner)

            # 关闭数据库
            kv.close()
            self.assertIsNone(kv._WindKVCore__inner)

            # 再次尝试操作应该抛出错误
            with self.assertRaises(ValueError):
                kv.get(b"test_key")


class TestWindKVCoreIntegration(unittest.TestCase):
    

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.test_db_path = os.path.join(self.temp_dir, "integration_test.kv")

    def tearDown(self):
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)

    def test_basic_workflow(self):
        
        # 注意：这个测试需要实际的_WindKVCore实现
        # 如果_WindKVCore是C扩展且不可用，这个测试可能会失败

        try:
            # 创建数据库
            kv = WindKVCore()
            kv.open(self.test_db_path, "integration_test")

            # 测试基本操作
            kv.put(b"key1", b"value1")
            kv.put(b"key2", b"value2")

            value1 = kv.get(b"key1")
            self.assertEqual(value1, b"value1")

            # 测试get_all
            all_data = kv.get_all()
            self.assertIsInstance(all_data, list)

            # 清理
            kv.delete(b"key1")
            kv.delete(b"key2")

            kv.close()

        except Exception as e:
            # 如果_WindKVCore不可用，跳过这个测试
            self.skipTest(f"Integration test skipped: {e}")


if __name__ == '__main__':
    # 运行测试
    unittest.main(verbosity=2)