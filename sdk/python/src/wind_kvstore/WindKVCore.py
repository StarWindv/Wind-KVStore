"""
Class `WindKVCore` encapsulated `wind_kvstore engine`'s ABI.

Usage:
``` Python
from wind_kvstore.WindKVCore import WindKVCore
```

"""


from typing import Optional, Union, List, Dict

from ._wind_kvcore import _WindKVCore
from functools import wraps
import os


def check_open_status():
    def decorator(func):
        @wraps(func)
        def wrapper(self, *args, **kwargs):
            if (hasattr(self, "_WindKVCore__inner")
                    and
                    getattr(self, "_WindKVCore__inner")
                    is None
            ):
                raise ValueError(
                    "The database is not opened."
                )
            return func(self, *args, **kwargs)
        return wrapper
    return decorator


class WindKVCore:
    def __init__(
            self,
            path: Optional[str] = None,
            db_identifier: Optional[str] = None
    ):
        self.path = None
        self.db_identifier = None
        self.__inner = None
        if path:
            self.open(path, db_identifier)

    def open(
            self,
            path: str,
            db_identifier: Optional[str] = None
    ):
        path = os.path.abspath(
            os.path.expandvars(
                os.path.expanduser(
                    path
                )
            )
        )
        self.__inner = _WindKVCore(path, db_identifier)
        self.path = path
        self.db_identifier = self.__inner.get_identifier()

    @check_open_status()
    def get(
            self,
            key: Union[bytes, bytearray]
    ) -> Optional[bytes]:
        return self.__inner.get(key)

    @check_open_status()
    def get_all(self) \
            -> List[
                Optional[
                    Dict[
                        str, str
                    ]
                ]
            ]:
        _data = self.__inner.get_all()
        return [{"key": k, "value": v} for d in _data for k, v in d.items()]


    @check_open_status()
    def put(
            self,
            key: Union[bytes, bytearray],
            value: Union[bytes, bytearray]
    ) -> None:
        return self.__inner.put(key, value)

    @check_open_status()
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
        return self.__inner.delete(key)

    @check_open_status()
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
        return self.__inner.set_identifier(identifier)

    @check_open_status()
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
        return self.__inner.compact()

    @check_open_status()
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
        self.db_identifier = self.__inner.get_identifier()
        return self.db_identifier

    @check_open_status()
    def close(self):
        """
        关闭数据库，确保所有数据都已持久化。

        Closes the database, ensuring all data is persisted.

        Raises:
            PyIOError: 当IO操作失败时抛出
                       Raised when IO operation fails
        """
        self.__inner.close()
        self.__inner = None

    def __del__(self):
        if self.__inner is not None:
            self.close()

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.__inner is not None:
            self.close()

    def __str__(self):
        return f"WindKVCore(path={self.path}, db_identifier={self.db_identifier})"

    def __repr__(self):
        return f"WindKVCore(path={self.path}, db_identifier={self.db_identifier})"

    def __eq__(self, other):
        return (
                isinstance(other, WindKVCore)
                and self.path == other.path
                and self.db_identifier == other.db_identifier
        )
