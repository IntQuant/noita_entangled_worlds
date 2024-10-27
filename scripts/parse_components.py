import shlex
import json

def parse_component(component):
    it = iter(component)
    name = next(it)
    fields = []
    for line in it:
        line = line.strip()
        if line.startswith("-"):
            continue
        typ, name, *range_info, desc = shlex.split(line)
        fields.append({
            "field": name,
            "typ": typ,
            "desc": desc,
        })
        #print(name, typ, desc, range_info)
    return {
        "name": name,
        "fields": fields,
    }
    

path = "/home/quant/.local/share/Steam/steamapps/common/Noita/tools_modding/component_documentation.txt"

components = []
current = []
for line in open(path):
    if line == "\n":
        if current:
            components.append(current)
            current = []
    else:
        current.append(line)

assert not current

parsed = [parse_component(component) for component in components]
json.dump(parsed, open("components.json", "w"), indent=None)

