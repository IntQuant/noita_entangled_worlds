import xml.etree.ElementTree as ET
import os
import json

base_path = "scripts/gdb_data/"
os.makedirs(base_path, exist_ok=True)

source_path = "/home/quant/noita.exe.xml"
tree = ET.parse(source_path)
root = tree.getroot()

vtables = dict()

for symbol in root.find('SYMBOL_TABLE'):
    addr = int(symbol.attrib["ADDRESS"], base=16)
    if symbol.attrib["NAME"] == "vftable":
        vtables[addr] = symbol.attrib["NAMESPACE"]

json.dump(vtables, open(base_path+"vtables.json", "w"))
