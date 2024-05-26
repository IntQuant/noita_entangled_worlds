with open("hex.lua", "w") as f:
    print("return {", file=f)
    for i in range(256):
        print(f'"{hex(i)[2:].rjust(2, "0")}",', file=f)
    print("}", file=f)