import subprocess
from zipfile import ZipFile, ZIP_DEFLATED as COMPRESS_TYPE
import shutil
import os
import tomllib
import json
from dataclasses import dataclass

cargo_manifest = tomllib.load(open("noita-proxy/Cargo.toml", "rb"))
version = cargo_manifest["package"]["version"]

print("Current version: ", version)

COMPRESS_LEVEL = 9

@dataclass
class PullRequest:
    number: int
    author: str
    title: str

    def __str__(self):
        return f"{self.title} by @{self.author} in #{self.number}"

def try_remove(path):
    try:
        os.remove(path)
    except FileNotFoundError:
        pass

class ReleaseNotes:
    def __init__(self):
        self.md = []

    def title(self, t):
        self.md.append(f"## {t}\n")

    def p(self, t):
        self.md.append(f"\n{t}\n")

    def l(self, t):
        self.md.append(f"- {t}")

    def gen_md(self):
        return "\n".join(self.md)

def call_parse_json(args):
    return json.loads(subprocess.check_output(args))

def check_release_exists(tag):
    print("Checking if release exists:", tag)
    ret = subprocess.call(["gh", "release", "view", tag])
    exists = ret == 0
    return exists

def get_last_release():
    return call_parse_json(["gh", "release", "view", "--json", "publishedAt,name"])

def get_pull_requests_from(date):
    parsed = call_parse_json(["gh", "pr", "list", "--state", "merged", "--search", "merged:>"+date, "--json", "number,title,author"])
    return [PullRequest(entry["number"], entry["author"]["login"], entry["title"]) for entry in parsed]

def extract_steam_redist():
    os.makedirs("target/tmp", exist_ok=True)

def make_release_assets():
    print("Compiling Noita Proxy...")

    os.chdir("noita-proxy")

    subprocess.run(["cross", "build", "--profile", "release-lto", "--target", "x86_64-unknown-linux-gnu"], check=True)
    subprocess.run(["cargo", "build", "--release", "--target", "x86_64-pc-windows-gnu"], check=True)

    os.chdir("..")

    os.makedirs("target", exist_ok=True)
    os.makedirs("target/tmp", exist_ok=True)

    try_remove("target/noita-proxy-win.zip")
    try_remove("target/noita-proxy-linux.zip")
    try_remove("target/quant.ew.zip")

    print("Extracting steam dylib...")

    extract_steam_redist()

    print("Writing win release...")

    with ZipFile("target/noita-proxy-win.zip", "w") as release:
        release.write("noita-proxy/target/x86_64-pc-windows-gnu/release/noita-proxy.exe", arcname="noita_proxy.exe", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)
        release.write("redist/steam_api64.dll", arcname="steam_api64.dll", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)

    print("Writing linux release...")

    with ZipFile("target/noita-proxy-linux.zip", "w") as release:
        release.write("noita-proxy/target/x86_64-unknown-linux-gnu/release-lto/noita-proxy", arcname="noita_proxy.x86_64", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)
        release.write("redist/libsteam_api.so", arcname="libsteam_api.so", compress_type=COMPRESS_TYPE, compresslevel=COMPRESS_LEVEL)

    print("Writing mod release...")

    shutil.make_archive("target/quant.ew", "zip", "quant.ew")

    with ZipFile("target/quant.ew.zip", "a") as release:
        release.writestr("files/version.lua", f'return "{version}"')

def main():
    tag = "v"+version
    if check_release_exists(tag):
        print("Release already exists, exiting")
        exit(1)

    subprocess.run(["git", "pull"])
    subprocess.run(["git", "commit", "-am", "Automated commit: "+tag])
    subprocess.run(["git", "push"])

    last_release = get_last_release()
    print("Last release is:", last_release["name"])

    pull_requests = get_pull_requests_from(last_release["publishedAt"])
    print()

    make_release_assets()

    print()
    print("Will release:", tag)
    print("Accepted pull requests:")

    for request in pull_requests:
        print(request)

    print()

    notes = ReleaseNotes()

    notes.title("Noita Entangle Worlds "+tag)

    notes.p("")

    notes.title("Accepted pull requests")
    if pull_requests:
        for request in pull_requests:
            notes.l(request)
    else:
        notes.p("No pull requests have been accepted in this release.")

    notes.title("Installation")
    notes.p("Download and unpack `noita-proxy-win.zip` or `noita-proxy-linux.zip`, depending on your OS. After that, launch the proxy.")
    notes.p("Proxy is able to download and install the mod automatically. There is no need to download the mod (`quant.ew.zip`) manually.")
    notes.p("""You'll be prompted for a path to `noita.exe` when launching the proxy for the first time.
It should be detected automatically as long as you use steam version of the game and steam is launched.
        """)

    notes.title("Updating")
    notes.p("There is a button in bottom-left corner on noita-proxy's main screen that allows to auto-update to a new version when one is available")

    print()
    notes_path = "/tmp/rnotes.md"
    with open(notes_path, "w") as f:
        print(notes.gen_md(), file=f)

    subprocess.check_call(["nano", notes_path])

    title = input("Release title: ")

    subprocess.call(["gh", "release", "create", tag, "--title", f"{tag} - {title}", "-F", notes_path, "./target/noita-proxy-linux.zip", "./target/noita-proxy-win.zip", "./target/quant.ew.zip"])

if __name__ == "__main__":
    main()
