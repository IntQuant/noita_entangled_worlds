import subprocess
from zipfile import ZipFile, ZIP_DEFLATED as COMPRESS_TYPE
import shutil
import os

def try_remove(path):
    try:
        os.remove(path)
    except FileNotFoundError:
        pass

COMPRESS_LEVEL = 9

print("Compiling Noita Proxy...")

os.chdir("noita-proxy")

subprocess.run(["cargo", "build", "--release"])
subprocess.run(["cargo", "build", "--release", "--target", "x86_64-pc-windows-gnu"])

os.chdir("..")

os.makedirs("target", exist_ok=True)
os.makedirs("target/tmp", exist_ok=True)

try_remove("target/noita-proxy-win.zip")
try_remove("target/noita-proxy-linux.zip")
try_remove("target/quant.ew.zip")

print("Extracting steam dylib...")

with ZipFile("redist/steam_dylib.zip", "r") as steam_dylib_zip:
    steam_dylib_zip.extractall("target/tmp")

print("Writing win release...")

with ZipFile("target/noita-proxy-win.zip", "w") as release:
    release.write("noita-proxy/target/x86_64-pc-windows-gnu/release/noita-proxy.exe", arcname="noita_proxy.exe", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)
    release.write("target/tmp/steam_api64.dll", arcname="steam_api64.dll", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)

print("Writing linux release...")

with ZipFile("target/noita-proxy-linux.zip", "w") as release:
    release.write("noita-proxy/target/release/noita-proxy", arcname="noita_proxy.x86_64", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)
    release.write("target/tmp/libsteam_api.so", arcname="libsteam_api.so", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)

print("Writing mod release...")

shutil.make_archive("target/quant.ew", "zip", "quant.ew")
