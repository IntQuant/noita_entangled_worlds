import gdb
import json

patches = dict()

class PatchNop(gdb.Command):
  def __init__ (self):
    super().__init__ ("patch-nop", gdb.COMMAND_USER)

  def invoke(self, arg, from_tty):
    addr = int(arg, base=0)
    if addr in patches:
        print("Already patched")
        return
    print("Patching", hex(addr))
    inferior = gdb.selected_inferior()
    ln = 5
    mem = inferior.read_memory(addr, ln)
    print("Current memory", mem.hex())
    patches[addr] = mem
    inferior.write_memory(addr, bytes([0x90])*ln)
    print("Ok")

class PatchRevert(gdb.Command):
  def __init__ (self):
    super().__init__ ("patch-revert", gdb.COMMAND_USER)

  def invoke(self, arg, from_tty):
    inferior = gdb.selected_inferior()
    for addr, mem in patches.items():
        print("Restoring", hex(addr))
        inferior.write_memory(addr, mem)
    patches.clear()

_vtables_cache = None

def get_vtables():
  global _vtables_cache
  if _vtables_cache is None:
    print("Loading VTables")
    _vtables_cache = {}
    for k, v in json.load(open("gdb_data/vtables.json")).items():
      _vtables_cache[int(k)] = v
  return _vtables_cache

class MemReader:
  def __init__(self):
    self.inferior = gdb.selected_inferior()
  
  def read_uint(self, addr: int):
    try:
      return int.from_bytes(self.inferior.read_memory(addr, 4), byteorder="little", signed=False)
    except gdb.MemoryError:
      return None

class ValueIdentifier:
  def __init__(self, addr: int):
    self.addr = addr
    self.mem = MemReader()
    self.vtables = get_vtables()
    self.type = None
    self.name = None
    self.identify()
  
  def identify(self):
    if self.addr == 0:
      self.type = "null"
      return
    if self.addr in self.vtables:
        self.type = "pointer to vtable"
        self.name = self.vtables[self.addr]
        return
    val = self.mem.read_uint(self.addr)
    if val is None:
      self.type = "unreadable pointer"
    else:
      #print(f"Checking if {hex(val)} is a vtable")
      if val in self.vtables:
        self.type = "pointer to value of type"
        self.name = self.vtables[val]
        return

class Identify(gdb.Command):
  def __init__ (self):
    super().__init__ ("identify", gdb.COMMAND_USER)

  def invoke(self, arg, from_tty):
    addr = int(arg, base=0)
    vi = ValueIdentifier(addr)
    print(vi.type, vi.name)

class Layout(gdb.Command):
  def __init__ (self):
    super().__init__ ("layout", gdb.COMMAND_USER)

  def invoke(self, arg, from_tty):
    splt = arg.split()
    addr = int(splt[0], base=0)
    amount = 1024
    if len(splt) > 1:
      amount = int(splt[1])
    mem = MemReader()
    for i in range(amount):
      p = mem.read_uint(addr+i*4)
      if p is not None:
        vi = ValueIdentifier(p)
        if vi.type is not None and vi.type != "null" and vi.type != "unreadable pointer":
          print(f"Field {hex(i*4)} ({hex(addr+i*4)}, {hex(p)})", vi.type, vi.name)


PatchNop()
PatchRevert()
Identify()
Layout()
