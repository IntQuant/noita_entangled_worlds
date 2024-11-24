import json

path = "/home/quant/.local/share/Steam/steamapps/common/Noita/tools_modding/lua_api_documentation.html"

lines = open(path).readlines()

lines_iter = iter(lines)

parsed = []

def maybe_map_types(name, typ):
    if name == "entity_id":
        typ = "entity_id"
    if name == "component_id":
        typ = "component_id"
    if typ == "float":
        typ = "number"
    if typ == "uint":
        typ = "color"
    if typ == "uint32":
        typ = "color"
    return typ

def parse_arg(arg_s):
    if "|" in arg_s:
        raise ValueError("multiple argument types not supported")
    if "{" in arg_s:
        raise ValueError("no table support for now")
    if "multiple_types" in arg_s:
        raise ValueError("no 'multiple_types' either")
    other, *default = arg_s.split("=", maxsplit=1)
    other = other.strip()
    if default:
        default = default[0].strip()
    else:
        default = None
    name, typ = other.split(":", maxsplit=1)

    typ = maybe_map_types(name, typ)

    return {
        "name": name,
        "typ": typ,
        "default": default,
    }

def parse_ret(ret_s):
    if not ret_s:
        return None
    
    optional = ret_s.endswith("|nil")
    ret_s = ret_s.removesuffix("|nil")
    
    if "|" in ret_s:
        raise ValueError("multiple return types not supported")
    if "{" in ret_s:
        raise ValueError("tables in returns not supported")
    if "multiple_types" in ret_s:
        raise ValueError("no 'multiple_types' either")
    
    typ = ret_s
    name = None
    if ":" in ret_s:
        name, typ = ret_s.split(":", maxsplit=1)

    typ = maybe_map_types(name, typ)

    return {
        "name": name,
        "typ": typ,
        "optional": optional
    }
    

ignore = {
    "PhysicsApplyForceOnArea",
    "GetRandomActionWithType",
}
skipped = 0
deprecated = 0

# 2 lazy 2 parse xml properly
try:
    while True:
        line = next(lines_iter)
        if line.startswith('<th><span class="function">'):
            fn_line = line.strip()
            ret_line = next(lines_iter).strip()
            desc_line = next(lines_iter).strip()

            fn_name, other = fn_line.removeprefix('<th><span class="function">').split('</span>(<span class="field_name">', maxsplit=1)
            args = other.removesuffix('</span><span class="func">)</span></th>').strip().split(", ")
            try:
                args = [parse_arg(arg) for arg in args if ":" in arg]
            except ValueError as e:
                skipped += 1
                print(f"Skipping {fn_name}: {e}")
                continue


            rets = ret_line.removeprefix('<th><span class="field_name">').removesuffix('</span></th></th>').strip()
            desc = desc_line.removeprefix('<th><span class="description">').removesuffix('</span></th></th>').strip().replace("</br>", "\n")

            if "Debugish" in rets:
                rets, desc = rets.split(" (", maxsplit=1)
                desc = desc.removesuffix(")")
            rets = rets.split(", ")
            try:
                rets = [parse_ret(ret) for ret in rets if ret]
            except ValueError as e:
                print(f"Skipping {fn_name}: {e}")
                skipped += 1
                continue
            if not desc:
                desc = "Nolla forgot to include a description :("

            if "Deprecated" in desc_line:
                deprecated += 1
                print(f"Skipping {fn_name}: deprecated")
                continue

            #print(fn_line, ret_line, desc_line)
            
            if fn_name not in ignore:
                #print(fn_name, args, "->", ret)
                parsed.append({
                    "fn_name": fn_name,
                    "args": args,
                    "desc": desc,
                    "rets": rets
                })
            else:
                skipped += 1

except StopIteration:
    pass


print("Total skipped:", skipped, deprecated)
print("Total parsed:", len(parsed))
json.dump(parsed, open("ewext/noita_api_macro/src/lua_api.json", "w"), indent=2)
