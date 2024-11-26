import re

cargo_manifest = open("noita-proxy/Cargo.toml", "r").read()
version = re.findall('version = "(.*?)"', cargo_manifest)[0]

print("Version:", version)
assert version is not None
