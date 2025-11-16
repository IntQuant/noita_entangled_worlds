import gdb

patches = dict()

class PatchNop(gdb.Command):
  def __init__ (self):
    super().__init__ ("patch-nop", gdb.COMMAND_USER)

  def invoke (self, arg, from_tty):
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

  def invoke (self, arg, from_tty):
    inferior = gdb.selected_inferior()
    for addr, mem in patches.items():
        print("Restoring", hex(addr))
        inferior.write_memory(addr, mem)
    patches.clear()


PatchNop()
PatchRevert()
