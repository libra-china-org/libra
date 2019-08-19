import binascii
import json
from ctypes import *

def new_strbuff(sz=4096):
  return create_string_buffer(b'\000' * sz), c_int(sz)

def prettyjson_from_buff(buff, sz):
  data = buff.value[:sz.value]
  return json.dumps(json.loads(data), indent=2)

lib = cdll.LoadLibrary('../target/debug/libffibridge.dylib')

r_buff, r_size = new_strbuff()

# get_allowed_scripts
r = lib.get_allowed_scripts(r_buff, byref(r_size))
print('get_allowed_scripts:', r, r_size,
      prettyjson_from_buff(r_buff, r_size))

# decode AccountState blob
blob = binascii.a2b_hex('010000002100000001217da6c6b3e19f1825cfb2676daecce3bf3de03cf26647c78df00b371b25cc974500000020000000e94f428835ac0ef564a6889954158be87be1cd2198e79f96bbb25d9b30b70943c05d634e1809000000010000000000000000000000000000000000000000000000')
blob_size = len(blob)
r = lib.decode_account_state_blob(blob, blob_size, r_buff, byref(r_size))
print('decode_account_state_blob:', r, r_size,
      prettyjson_from_buff(r_buff, r_size))
