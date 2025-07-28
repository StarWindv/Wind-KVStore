import json
import re
import time
import warnings
from typing import List

import requests


def remind(msg: str)->None:
    if not msg:
        raise ValueError(
            f"The warning msg can not be empty."
        )
    warnings.warn(
        msg,
        DeprecationWarning,
        stacklevel=2
    )


def format_exec_response(response_text: str)->json:
    cleaned_text = response_text.strip('"').replace('\\"', '"')
    parts = cleaned_text.split('" "')
    result = {}
    for i, part in enumerate(parts, 1):
        part = part.strip('"')
        if ': ' in part:
            cmd_part, msg_part = part.rsplit(': ', 1)
            cmd = cmd_part.rstrip(';')
            if i == 1:
                cmd = cmd.replace('{\"status\":\"\"\\n    ', '').strip()
            if ": Error" not in cmd:
                result[f"cmd{i}"] = {
                    "command": cmd.lstrip(),
                    "message": msg_part.strip("'")
                }
    return json.dumps(result, indent=2)


def format_exec_put_command(input_str: str)->str:
    pattern = r'(PUT\s*"[^"]+")\s*:\s*("[^"]+")'
    result = re.sub(pattern, r'\1:\2', input_str)
    return result


class _ConnectKV:
    _DATA_ROUTES = {
        "GET": "/api/get",
        "PUT": "/api/put",
        "DEL": "/api/del"
    }
    _KV_ROUTES = {
        "OPEN": "/api/open",
        "CLOSE": "/api/close",
        "CURRENT": "/api/current"
    }
    _MANAGEMENT_ROUTES = {
        "COMPACT": "/api/compact"
    }
    _ID_ROUTES = {
        "GET_ID": "/api/id/get",
        "SET_ID": "/api/id/set"
    }
    _EXEC_ROUTES = {
        "EXECUTE": "/api/execute"
    }

    def __init__(self, host: str, port: int):
        """
        I recommend that you use more
        class attributes instead of
        definitions obtained through _map,
        as readability is not good

        :param host         : string : server host.
        :param port         : int    : server port.
        """
        self.host = host
        self.port = port
        self._map()

    def _map(self):
        self.get_method = {
            "CLOSE"  : self._KV_ROUTES["CLOSE"],
            "CURRENT": self._KV_ROUTES["CURRENT"],
            "GET"    : self._DATA_ROUTES["GET"],
            "COMPACT": self._MANAGEMENT_ROUTES["COMPACT"],
            "GET_ID" : self._ID_ROUTES["GET_ID"]
        }

        self.post_method = {
            "OPEN"   : self._KV_ROUTES["OPEN"],
            "PUT"    : self._DATA_ROUTES["PUT"],
            "DEL"    : self._DATA_ROUTES["DEL"],
            "SET_ID" : self._ID_ROUTES["SET_ID"],
            "EXECUTE": self._EXEC_ROUTES["EXECUTE"]
        }


class _WindKVStoreBase(_ConnectKV):
    def __init__(
            self,
            host: str = "127.0.0.1",
            port: int = 14514,
            kv_path: str = "",
            use_https: bool = False,
            check_active=True,
            use_time_to_check_active=False
    ):
        """
        :param host         : string : server host.
        :param port         : int    : server port.
        :param kv_path      : string : the database's path which you want to use.
        :param use_https    : bool   : Whether to enable HTTPS connection.
        :param check_active : bool   : Check if the database is active before each operation
        :param use_time_to_check_active: bool: Use timestamps to locally detect whether database sessions are active
        """
        super().__init__(host, port)
        self.session_id = ""
        self.kv_path = kv_path

        self.session_start = 0.0
        self.protocol = "https" if use_https else "http"
        self.check_active = check_active
        self.use_time_to_check_active = use_time_to_check_active
        if self.use_time_to_check_active:
            remind("""
        When using timestamps for active detection, 
        database sessions that cannot detect anomalies will be closed. 
        It is recommended to use the general method.
        """)

        if self.kv_path:
            self._open(self, self.kv_path)

    @classmethod
    def _open(cls, self, path) -> bool:
        """
        method: POST
        :param path: The database path which you want to use.
        :type  path: str
        :return: bool
        """
        data = {
            "path": path
        }
        headers = {
            "Content-Type": "application/json"
        }
        # print(self.protocol)
        url = f"{self.protocol}://{self.host}:{self.port}{cls._KV_ROUTES["OPEN"]}"
        response = requests.post(url, json=data, headers=headers)
        for key, value in response.headers.items():
            if key.lower() == "x-session-id":
                self.session_id = value
                self.session_start = time.time()
                return True
        return False

    @classmethod
    def _close(cls, self) -> json:
        """
        method: GET
        :return: json
        """
        if not self.session_id:
            raise ValueError("No kv-store opened.")
        headers = {
            "Content-Type": "application/json",
            "X-Session-ID": self.session_id
        }
        url = f"{self.protocol}://{self.host}:{self.port}{cls._KV_ROUTES["CLOSE"]}"
        response = requests.get(url, headers=headers)
        return response.json()

    @classmethod
    def _get_current_path(cls, self) -> str or None:
        """
        method: GET
        :return: str | None
        """
        if not self.session_id:
            # print(f"\033[96m - session_id: {self.session_id}\033[0m")
            raise ValueError("No kv-store opened.")
        headers = {
            "Content-Type": "application/json",
            "X-Session-ID": self.session_id
        }
        url = f"{self.protocol}://{self.host}:{self.port}{cls._KV_ROUTES["CURRENT"]}"
        response = requests.get(url, headers=headers).json()
        return response.get("path", None)

    @classmethod
    def _get_value(cls, self, key: str):
        """
        method: GET
        :param key: The key which you want to query.
        :return: json
        """
        headers = {
            "X-Session-ID": self.session_id
        }
        url = f"{self.protocol}://{self.host}:{self.port}{self._DATA_ROUTES["GET"]}?key={key}"
        response = requests.get(url, headers=headers)
        return response.json()

    @classmethod
    def _put_kv(cls, self, key: str, value: str) -> json:
        """
        method: POST
        """
        data = json.dumps([{
            "key": key,
            "value": value
        }])
        # print(data)
        headers = {
            "Content-Type": "application/json",
            "X-Session-ID": self.session_id
        }
        url = f"{self.protocol}://{self.host}:{self.port}{cls._DATA_ROUTES["PUT"]}"
        response = requests.post(url, data=data, headers=headers)
        # print(response.text)
        return response.json()

    @classmethod
    def _del_kv(cls, self, key: str) -> json:
        """
        method: POST
        :param key: The key-value's key which you want to delete.
        :return: json
        """
        headers = {
            "X-Session-ID": self.session_id,
            "Content-Type": "application/json"
        }
        data = {
            "key": key
        }
        url = f"{self.protocol}://{self.host}:{self.port}{self._DATA_ROUTES["DEL"]}"
        response = requests.post(url, headers=headers, json=data)
        # print(response.text)
        return response.json()

    @classmethod
    def _compact(cls, self) -> json:
        """
        method: GET
        :return: json
        """
        headers = {
            "X-Session-ID": self.session_id
        }
        url = f"{self.protocol}://{self.host}:{self.port}{self._MANAGEMENT_ROUTES["COMPACT"]}"
        response = requests.get(url, headers=headers)
        return response.json()

    @classmethod
    def _get_kv_id(cls, self)->json:
        """
        method: GET
        :return: json
        """
        headers = {
            "X-Session-ID": self.session_id
        }
        url = f"{self.protocol}://{self.host}:{self.port}{self._ID_ROUTES["GET_ID"]}"
        response = requests.get(url, headers=headers)
        return response.json()

    @classmethod
    def _set_kv_id(cls, self, identifier: str)->json:
        """
        method: POST
        :param identifier: The new identifier for current kv.
        :return: json
        """
        headers = {
            "X-Session-ID": self.session_id,
            "Content-Type": "application/json"
        }
        data = {
            "identifier": identifier
        }
        url = f"{self.protocol}://{self.host}:{self.port}{self._ID_ROUTES["SET_ID"]}"
        response = requests.post(url, headers=headers, json=data)
        return response.json()

    @classmethod
    def _execute(cls, self, command: str)->str:
        """
        server response like: \n
        curl http://localhost:14514/api/execute \n
        -H "X-Session-ID: 9a02d2ac-d91b-42b5-9935-8dc2e6cc3f5b" \n
        -H "content-type: text/plain" -d "PUT \"1\":\"test\"; IDENTIFIER SET \"111\";" \n
        {"status":
              "\"PUT \"1\":\"test\": Inserted 1 key-value pairs;\" \" IDENTIFIER SET \"111\": Identifier set to '111'\""}

        method: POST
        :param command: The key-value's key which you want to delete.
        :return: str
        """
        headers = {
            "X-Session-ID": self.session_id,
            "Content-Type": "text/plain"
        }
        data = format_exec_put_command(command) # Important!
        url = f"{self.protocol}://{self.host}:{self.port}{self._EXEC_ROUTES["EXECUTE"]}"
        response = requests.post(url, headers=headers, data=data)
        return response.text


class WindKVStore(_WindKVStoreBase):
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()

    def __init__(
            self,
            host: str = "127.0.0.1",
            port: int = 14514,
            kv_path: str = "",
            use_https: bool = False,
            check_active=True,
            use_time_to_check_active=False
    ):
        super().__init__(
                        host,
                        port,
                        kv_path,
                        use_https,
                        check_active,
                        use_time_to_check_active
        )

    @staticmethod
    def _check_activate():
        def middle(func):
            def wrapper(self, *args, **kwargs):
                if self.check_active:
                    if self.use_time_to_check_active:
                        if time.time() - float(self.session_start) > 1800:
                            self.open(self.kv_path)
                            # print("pass")
                    elif not self.is_activate():
                        self.open(self.kv_path)
                        # print("pass")
                return func(self, *args, **kwargs)
            return wrapper
        return middle

    def open(self, path) -> bool:
        if not path:
            raise ValueError(
                f"The arg `path` can not be empty. "
            )
        self.kv_path = path
        return self._open(self, path)

    def close(self) -> bool:
        if "open" in self._close(self).get("status"):
            # """
            # server return:
            # case 1: 'No database open'
            # case 2: 'Database closed'
            # """
            return False
        return True

    def get_current_path(self) -> str or None:
        return self._get_current_path(self)

    def is_activate(self) -> True:
        if not self._get_current_path(self):
            return False
        return True

    @_check_activate()
    def put_kv(self, key: str, value: str) -> bool:
        if not all([key, value]):
            raise ValueError(
                f"The key and value can not be empty.(key: {key}, value: {value})"
            )

        return "Inserted" in self._put_kv(self, key, value).get("status")

    @_check_activate()
    def get_value(self, key: str) -> str:
        return self._get_value(self, key).get("value")

    @_check_activate()
    def del_kv(self, key: str) -> bool:
        """
        To be honest,
        Unless there is a server error,
        the route `/api/del` always returns
        {"status": "Key deleted"}
        :param key: The key-value's key which you want to delete.
        :return: bool
        """
        if not key:
            raise ValueError(
                f"The function `del_kv`'s param `key` can not be empty."
            )
        if "deleted" in self._del_kv(self, key).get("status"):
            return True
        return False

    @_check_activate()
    def compact(self) -> json:
        """
        Compact your key-value database.
        :return: bool
        """
        if "compacted" in self._compact(self).get("status"):
            return True
        return False

    def change_kv(self, path: str) -> bool:
        """
        Replace the database pointed to
        by the current object with a new one.

        :param path: The new database path which you want to use
        :return: bool
        """
        if not path:
            raise ValueError(
                f"The function's param `path` can not be empty."
            )
        if not isinstance(path, str):
            raise TypeError(
                f"The function's param `path` must be `str`.(with type: {type(path)})"
            )
        self.close()
        self.kv_path = path
        return self.open(self.kv_path)

    @_check_activate()
    def get_kv_id(self) -> str | None:
        return self._get_kv_id(self).get("identifier")

    @_check_activate()
    def set_kv_id(self, identifier: str) -> bool:
        if not identifier:
            raise ValueError(
                f"The function `set_kv_id`'s param `identifier` can not be empty."
            )
        return "updated" in self._set_kv_id(self, identifier).get("status")

    @_check_activate()
    def execute(self, command: str) -> str:
        if not command:
            raise ValueError(
                f"The function `execute`s' param `command` can not be empty."
            )

        return format_exec_response(self._execute(self, command))

    @_check_activate()
    def state_execute(self, command: str) -> List[bool | str] | None:
        if not command:
            raise ValueError(
                f"The function `execute`s' param `command` can not be empty."
            )
        exec_result = json.loads(format_exec_response(self._execute(self, command)))
        result = []
        for sub_dict in exec_result.values():
            # print(sub_dict)
            c_msg = sub_dict.get("command")
            m_msg = sub_dict.get("message")
            if c_msg.startswith("PUT"):
                result.append("Inserted" in m_msg)
            elif c_msg.startswith("DEL"):
                result.append("deleted" in m_msg)
            elif c_msg.startswith("GET"):
                result.append("not found" not in m_msg)
            elif c_msg.startswith("IDENTIFIER SET"):
                result.append("set to" in m_msg)
            elif c_msg.startswith("IDENTIFIER GET"):
                result.append("PASS(ID-GET)")
            elif c_msg.startswith("COMPACT"):
                result.append("compacted" in m_msg)
        return result
