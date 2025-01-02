---Native library. Primarily for internal use.
---@module 'noitapatcher.nsew.native_dll'

local ffi = require("ffi")

native_dll = {}

---The NSEW support dll loaded in with `ffi.load`.
native_dll.lib = ffi.load(__nsew_path .. "nsew_native.dll")

return native_dll
