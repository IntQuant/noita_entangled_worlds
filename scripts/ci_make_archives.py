import tomllib
import os
from zipfile import ZipFile, ZIP_DEFLATED as COMPRESS_TYPE
import shutil

COMPRESS_LEVEL = 9

cargo_manifest = tomllib.load(open("noita-proxy/Cargo.toml", "rb"))
version = cargo_manifest["package"]["version"]

os.makedirs("target", exist_ok=True)

with ZipFile("target/noita-proxy-win.zip", "w") as release:
    release.write("noita-proxy/target/x86_64-pc-windows-gnu/release/noita-proxy.exe", arcname="noita_proxy.exe", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)
    release.write("redist/steam_api64.dll", arcname="steam_api64.dll", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)

print("Writing linux release...")

with ZipFile("target/noita-proxy-linux.zip", "w") as release:
    release.write("noita-proxy/target/release/noita-proxy", arcname="noita_proxy.x86_64", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)
    release.write("redist/libsteam_api.so", arcname="libsteam_api.so", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)

print("Writing mod release...")

shutil.make_archive("target/quant.ew", "zip", "quant.ew")

with ZipFile("target/quant.ew.zip", "a") as release:
    release.writestr("files/version.lua", f'return "{version}"')
