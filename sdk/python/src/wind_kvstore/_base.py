class ConnectKV:
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