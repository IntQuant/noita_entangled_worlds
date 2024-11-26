from prepare_release import *

def generate_notes(tag):
    last_release = get_last_release()
    print("Last release is:", last_release["name"])

    pull_requests = get_pull_requests_from(last_release["publishedAt"])

    notes = ReleaseNotes()

    notes.title("Noita Entangled Worlds "+tag)

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

    return notes_path

def main():
    tag = "v"+version
    if check_release_exists(tag):
        print("Release already exists, exiting")
        exit(1)

    subprocess.run(["git", "pull"])
    subprocess.run(["git", "commit", "-am", "Automated commit: "+tag])

    notes_path = generate_notes(tag)
    
    subprocess.check_call(["git", "tag", "-a", "-F", notes_path, tag ])
    subprocess.check_call(["git", "push"])


if __name__ == "__main__":
    main()
