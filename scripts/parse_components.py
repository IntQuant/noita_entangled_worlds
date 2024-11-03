import shlex
import json

all_types = set()

renames = {
    "std_string": "std::string",
}

def parse_component(component):
    it = iter(component)
    c_name = next(it)
    c_name = c_name.strip("\n")
    if "-" in c_name or "\n" in c_name:
        print(component)
        exit(-1)
    fields = []
    
    for line in it:
        line = line.strip()
        if line.startswith("-"):
            continue
        typ, name, *range_info, desc = shlex.split(line)
        name = name.strip("\n")
        if name == "-":
            print(f"Field of type {typ} skipped")
            continue
        typ = renames.get(typ, typ)
        fields.append({
            "field": name,
            "typ": typ,
            "desc": desc,
        })
        all_types.add(typ)
        #print(name, typ, desc, range_info)
    return {
        "name": c_name,
        "fields": fields,
    }
    

path = "/home/quant/.local/share/Steam/steamapps/common/Noita/tools_modding/component_documentation.txt"

components = []
current = []

for i, line in enumerate(open(path)):
    if line == "\n":
        if current:
            components.append(current)
            current = []
    else:
        current.append(line)

assert not current

parsed = [parse_component(component) for component in components]
json.dump(parsed, open("ewext/noita_api_macro/src/components.json", "w"), indent=None)

#print(*all_types, sep="\n")

