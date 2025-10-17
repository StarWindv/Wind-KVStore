"""
wind_kvcore 模块提供了一个高效的键值存储引擎，用于持久化存储键值对数据。

The wind_kvcore module provides an efficient key-value storage engine for persistently storing key-value pairs.
"""

from typing import Optional, Union, List, Dict


class _WindKVCore:
    """
    键值存储核心类，提供数据的存储、读取、删除和管理功能。

    The core class for key-value storage, providing functionality for storing, reading,
    deleting and managing data.

    Attributes:
        inner: 内部KVStore实例，None表示已关闭
               Inner KVStore instance, None indicates closed state
    """

    def __init__(
            self,
            path: str,
            db_identifier: Optional[str] = None
    ) -> None:
        """
        打开或创建一个数据库。

        Opens or creates a database.

        Args:
            path: 数据库文件路径
                  Path to the database file
            db_identifier: 数据库标识（可选）
                           Database identifier (optional)

        Raises:
            PyIOError: 当文件操作失败时抛出
                       Raised when file operation fails
        """
        pass

    def get(
            self,
            key: Union[bytes, bytearray]
    ) -> Optional[bytes]:
        """
        根据键获取对应的值。

        Retrieves the value corresponding to the key.

        Args:
            key: 要查询的键（字节类型）
                 Key to query (bytes type)

        Returns:
            键对应的字节值，如果键不存在则返回None
            Byte value corresponding to the key, or None if the key does not exist

        Raises:
            PyIOError: 当IO操作失败时抛出
                       Raised when IO operation fails
            PyValueError: 当数据库已关闭时抛出
                          Raised when database is already closed
        """
        pass

    def get_all(self) \
            -> List[
                Optional[
                    Dict[
                        str, str
                    ]
                ]
            ]:
        """
        获取数据库中所有的键值对

        Retrieves all key-value pairs in the database

        Returns:
            列表, 其中存放的是字符串格式的键值对字典
            List containing dictionaries of key-value pairs in string format

        Raises:
            PyIOError: 当IO操作失败时抛出
                       Raised when IO operation fails
            PyValueError: 当数据库已关闭时抛出
                          Raised when database is already closed
        """
        pass

    def put(
            self,
            key: Union[bytes, bytearray],
            value: Union[bytes, bytearray]
    ) -> None:
        """
        存储或更新键值对。

        Stores or updates a key-value pair.

        Args:
            key: 要存储的键（字节类型）
                 Key to store (bytes type)
            value: 要存储的值（字节类型）
                   Value to store (bytes type)

        Raises:
            PyIOError: 当IO操作失败时抛出
                       Raised when IO operation fails
            PyValueError: 当数据库已关闭时抛出
                          Raised when database is already closed
        """
        pass

    def delete(
            self,
            key: Union[bytes, bytearray]
    ) -> None:
        """
        根据键删除对应的键值对。

        Deletes the key-value pair corresponding to the key.

        Args:
            key: 要删除的键（字节类型）
                 Key to delete (bytes type)

        Raises:
            PyIOError: 当IO操作失败时抛出
                       Raised when IO operation fails
            PyValueError: 当数据库已关闭时抛出
                          Raised when database is already closed
        """
        pass

    def set_identifier(self, identifier: str) -> None:
        """
        设置数据库的标识。

        Sets the identifier of the database.

        Args:
            identifier: 数据库标识字符串
                        Database identifier string

        Raises:
            PyIOError: 当IO操作失败时抛出
                       Raised when IO operation fails
            PyValueError: 当数据库已关闭时抛出
                          Raised when database is already closed
        """
        pass

    def compact(self) -> None:
        """
        压缩数据库，优化存储空间并提高性能。

        Compacts the database to optimize storage space and improve performance.

        Raises:
            PyIOError: 当IO操作失败时抛出
                       Raised when IO operation fails
            PyValueError: 当数据库已关闭时抛出
                          Raised when database is already closed
        """
        pass

    def close(self) -> None:
        """
        关闭数据库，确保所有数据都已持久化。

        Closes the database, ensuring all data is persisted.

        Raises:
            PyIOError: 当IO操作失败时抛出
                       Raised when IO operation fails
        """
        pass

    def get_identifier(self) -> str:
        """
        获取当前数据库的标识。

        Retrieves the identifier of the current database.

        Returns:
            数据库标识字符串
            Database identifier string

        Raises:
            PyValueError: 当数据库已关闭时抛出
                          Raised when database is already closed
        """
        pass
