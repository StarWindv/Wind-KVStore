__version__ = '0.0.3'
__license__ = "MIT"
__url__ = "https://github.com/StarWindv/Wind-KVStore/blob/main/sdk/python"


from auto import auto
import client
import pool
from WindKVCore import WindKVCore

cWindKVStore = client.WindKVStore
pWindKVStore = pool.WindKVStore

__all__ = [
    'auto',
    'cWindKVStore',
    'pWindKVStore',
    'WindKVCore'
]