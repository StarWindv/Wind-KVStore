from .utils import remind
import importlib


_VALID_CHOICES = ("client", "pool")
_DEFAULT = "client"
_MODULE = {
    "client": "wind_kvstore.client",
    "pool": "wind_kvstore.pool",
}
_CLASS_NAME = "WindKVStore"

def auto(chose: str):
    if chose not in _VALID_CHOICES:
        remind(
            f"""
                Not support chose: {chose}. (from `client`, `pool`)
                Use default chose: client.
                """
        )
        chose = _DEFAULT
    return getattr(importlib.import_module(_MODULE[chose]), _CLASS_NAME)