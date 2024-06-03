mats_path = "/home/quant/.local/share/Steam/steamapps/compatdata/881100/pfx/dosdevices/c:/users/steamuser/AppData/LocalLow/Nolla_Games_Noita/data/materials.xml"

import xml.etree.ElementTree as ET

tree = ET.parse(mats_path)

root = tree.getroot()

colors = []

mat_data = open("mat_data.txt", "w")

for i, child in enumerate(root):
    if child.tag not in ["CellData", "CellDataChild"]:
        continue
    name = child.get("name")
    print(i, name, file=mat_data)
    graphics = child.find("Graphics")
    if graphics is not None:
        color = graphics.get("color")
        if color is not None:
            colors.append(int(color[2:], 16))
        else:
            colors.append(0)
    else:
        colors.append(0)

with open("mat_colors.txt", "w") as f:
    print(*colors, sep=" ", file=f)