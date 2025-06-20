import os
import sys
from zipfile import ZipFile, ZIP_DEFLATED as COMPRESS_TYPE
import shutil

from ci_version import version

COMPRESS_LEVEL = 9

os.makedirs("target", exist_ok=True)

mode = sys.argv[1]

if mode == "windows":
    print("Writing windows release...")

    with ZipFile("target/noita-proxy-win.zip", "w") as release:
        release.write("noita-proxy/target/release/noita-proxy.exe", arcname="noita_proxy.exe", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)
        release.write("redist/steam_api64.dll", arcname="steam_api64.dll", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)
elif mode == "linux":
    print("Writing linux release...")

    with ZipFile("target/noita-proxy-linux.zip", "w") as release:
        release.write("noita-proxy/target/release/noita-proxy", arcname="noita_proxy.x86_64", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)
        release.write("redist/libsteam_api.so", arcname="libsteam_api.so", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)
elif mode == "macos":
    print("Writing macos release...")

    with ZipFile("target/noita-proxy-macos.zip", "w") as release:
        #release.write("noita-proxy/assets/Info.plist", arcname="noita_proxy.app/Contents/Info.plist", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)
        release.write("noita-proxy/target/release/noita-proxy", arcname="noita_proxy", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)
        release.write("redist/libsteam_api.dylib", arcname="libsteam_api.dylib", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)
elif mode == "mod":
    print("Writing mod release...")

    shutil.make_archive("target/quant.ew", "zip", "quant.ew")

    with ZipFile("target/quant.ew.zip", "a") as release:
        release.writestr("files/version.lua", f'return "{version}"')
else:
    exit(-1)