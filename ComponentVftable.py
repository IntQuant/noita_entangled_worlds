# TODO write a description for this script
# @author
# @category _Custom
# @keybinding
# @menupath
# @toolbar

import ghidra
from ghidra.app.decompiler.flatapi import FlatDecompilerAPI
from ghidra.app.script import GhidraState
from ghidra.app.util.cparser.C import CParser
from ghidra.program.flatapi import FlatProgramAPI
from ghidra.program.model.address import Address
from ghidra.program.model.data import (
    ArrayDataType,
    DataTypeConflictHandler,
    DataTypeManager,
    StringDataType,
    StructureDataType,
    CategoryPath,
)
from ghidra.program.model.listing import Program


def get_state():
    # type: () -> GhidraState
    return getState()


state = get_state()
program = state.getCurrentProgram()

fpapi = FlatProgramAPI(program)

fdapi = FlatDecompilerAPI(fpapi)


def hex_n(n):
    if n[0:2] == "0x":
        num = int(n, 16)
    else:
        num = int(n)
    return num


type_defs = {
    "int32": "int",
    "uint32": "uint",
    "uint32_t": "uint",
    "unsigned int": "uint",
    "int64": "longlong",
    "uint64": "ulonglong",
    "std::string": "StdString",
    "EntityID": "int",
    "int16": "short",
    "uint16": "ushort",
    "GAME_EFFECT::Enum": "GameEffect",
    "b2ObjectID": "b2Object*",

    "vec2": "Vec2",
    "LensValue<float>": "LensValueFloat",
    "LensValue<int>": "LensValueInt",
    "LensValue<bool>": "LensValueBool",
    "MAP_STRING_STRING": "MapStringString",
    "VEC_PENDINGPORTAL": "VecPendingPortal",
    "VECTOR_INT32": "VectorInt32",
    "VEC_NPCPARTY": "VecNpcParty",
    "VECTOR_STRING": "VectorString",
    "VEC_CUTTHROUGHWORLD": "VecCutThroughWorld",
    "grid::ICell": "Cell",
    "VERLET_TYPE::Enum": "VerletType",
    "UintArrayInline": "UintArrayInline",
    "FloatArrayInline": "FloatArrayInline",
    "Vec2ArrayInline": "Vec2ArrayInline",
    "VerletLinkArrayInline": "VerletLinkArrayInline",
    "VerletSprite": "VerletSprite",
    "ivec2": "Vec2i",
    "EntityID": "int",
    "types::aabb": "AABB",
    "ENTITY_VEC": "VecInt",
    "TeleportComponentState::Enum": "TeleportComponentState",
    "VEC_OF_MATERIALS": "VecMaterials",
    "VECTOR_FLOAT": "VecFloat",
    "VirtualTextureHandle": "VirtualTextureHandle",
    "SpriteStainsState": "SpriteStainsState",
    "SpriteStains": "SpriteStains",
    "ValueRange": "ValueRange",
    "types::fcolor": "Color",
    "as::Sprite": "Sprite",
    "SpriteRenderList": "SpriteRenderList",
    "STACK_ANIMATIONSTATE": "StackAnimationState",
    "ComponentTags": "ComponentTags",
    "StatusEffectType": "StatusEffectType",
    "ProjectileTriggers": "ProjectileTriggers",
    "VEC_ENTITY": "VecInt",
    "EntityTypeID": "EntityTypeID",
    "RAGDOLL_FX::Enum": "RagdollFx",
    "PROJECTILE_TYPE::Enum": "ProjectileType",
    "ConfigGunActionInfo": "ConfigGunActionInfo",
    "ConfigExplosion": "ConfigExplosion",
    "ConfigDamagesByType": "ConfigDamagesByType",
    "ConfigDamageCritical": "ConfigDamageCritical",
    "PixelSprite": "PixelSprite",
    "std::vector<b2Body*>*": "VecPtrB2Body",
    "b2WeldJoint": "b2WeldJoint",
    "types::xform": "xform",
    "JOINT_TYPE::Enum": "JointType",
    "b2Joint": "b2Joint",
    "b2Vec2": "b2Vec2",
    "b2Body": "b2Body",
    "PathFindingNodeHandle": "PathFindingNodeHandle",
    "VECTOR_PATHNODE": "VECTOR_PATHNODE",
    "PathFindingComponentState::Enum": "PathFindingComponentState",
    "PathFindingLogic": "PathFindingLogic",
    "MSG_QUEUE_PATH_FINDING_RESULT": "MsgQueuePathFindingResult",
    "PathFindingInput": "PathFindingInput",
    "ParticleEmitter_Animation": "ParticleEmitterAnimation",
    "PARTICLE_EMITTER_CUSTOM_STYLE::Enum": "ParticleEmitterCustomStyle",
    "NINJA_ROPE_SEGMENT_VECTOR": "VecNinjaRopeSegment",
    "MOVETOSURFACE_TYPE::Enum": "MoveToSurfaceType",
    "types::iaabb": "IAABB",
    "MATERIAL_VEC_DOUBLES": "VecDouble",
    "std::vector<int>": "VecInt",
    "LuaManager": "LuaManager",
    "ValueMap": "MapValue",
    "LUA_VM_TYPE::Enum": "LuaVmType",
    "ValueRangeInt": "ValueRangeInt",
    "ConfigLaser": "ConfigLaser",
    "INVENTORY_KIND::Enum": "InventoryKind",
    "ImGuiContext": "ImGuiContext",
    "INVENTORYITEM_VECTOR": "VecInventoryItem",
    "InvenentoryUpdateListener": "InvenentoryUpdateListener",
    "IKLimbStateVec": "IKLimbStateVec",
    "IKLimbAttackerState": "IKLimbAttackerState",
    "HIT_EFFECT::Enum": "HitEffect",
    "VISITED_VEC": "VecVisited",
    "USTRING": "UString",
    "VECTOR_STR": "VecStr",
    "VECTOR_ENTITYID": "VecInt",
    "EXPLOSION_TRIGGER_TYPE::Enum": "ExplosionTriggerType",
    "ConfigDrugFx": "ConfigDrugFx",
    "std::vector<int>": "StdVecInt",
    "std::vector<float>": "StdVecFloat",
    "CharacterStatsModifier": "CharacterStatsModifier",
    "AudioSourceHandle": "AudioSourceHandle",
    "DAMAGE_TYPES::Enum": "DamageTypes",
    "ARC_TYPE::Enum": "ArcType",
    "RtsUnitGoal": "RtsUnitGoal",
    "AI_STATE_STACK": "AIStateStack",
    "EntityTags": "EntityTags",
    "AIData": "AIData",
    "ceng::CArray2D<uint32>": "CArray2DUint32",
}


def get_types_file():
    file = askFile("Component Docs", "Approve").getAbsolutePath()
    content = open(file, "r").read()
    lines = content.replace("\r", "").split("\n")
    name = ""
    content = {
        "ParticleEmitterComponent": {
            "custom_style": "PARTICLE_EMITTER_CUSTOM_STYLE::Enum",
            "m_cached_image_animation": "ParticleEmitter_Animation*",
        },
        "ExplosionComponent": {"trigger": "EXPLOSION_TRIGGER_TYPE::Enum"},
        "InventoryComponent": {"update_listener": "InvenentoryUpdateListener*"},
        "PathFindingComponent": {
            "job_result_receiver": "MSG_QUEUE_PATH_FINDING_RESULT"
        },
    }
    for line in lines:
        if line == "":
            continue
        if line[0] != " ":
            name = line
            if name not in content.keys():
                content[name] = {}
            continue
        if line[1] == "-":
            continue
        parts = [p for p in line.strip().split(" ") if p != ""]
        ty = parts[0]
        field = parts[1]
        if ty in type_defs.keys():
            ty = type_defs[ty]
        content[name][field] = (ty, line[125:].replace('"', ""))
    return content


def do_vftable(addr, content, name):
    ref = [x.getFromAddress() for x in fpapi.getReferencesTo(addr)][0]
    fun = fpapi.getFunctionContaining(ref)
    super_parents = [
        fpapi.getFunctionContaining(x.getFromAddress())
        for x in fpapi.getReferencesTo(fun.getEntryPoint())
    ]
    size = None
    for super_parent in super_parents:
        super_parent_decomp = fdapi.decompile(super_parent)
        if "operator_new(" not in super_parent_decomp:
            continue
        if size is not None:
            continue
        size = hex_n(super_parent_decomp.split("operator_new(")[1].split(")")[0])

    parent = fdapi.decompile(fun)
    derived_size = False
    if size is None:
        if "operator_new(" in parent:
            size = hex_n(parent.split("operator_new(")[1].split(")")[0])
        else:
            derived_size = True
            size = 0x48
    new_addr = addr.add(14 * 4)
    v = hex(fpapi.getInt(new_addr))
    deref = fpapi.getAddressFactory().getAddress(v)
    decompiled = fdapi.decompile(fpapi.getFunctionAt(deref))
    things = []
    while True:
        data = {}
        found = decompiled.find('"')
        decompiled = decompiled[found + 1 :]
        if found == -1:
            break
        close = decompiled.find('"')
        if close == -1:
            print("no end found!")
            break
        if "{" not in decompiled[close + 1 :]:
            break
        data["field"] = str(decompiled[:close])
        decompiled = decompiled[close + 1 :]
        lines = decompiled.split("}")[0].split("{")[1].split("\n")
        for line in lines:
            if "+" in line:
                add = line.find("+")
                line = line[add + 2 :]
                num = line[:-1]
                num = hex_n(num)
                data["offset"] = num
            if "[2]" in line:
                assign = line.find("=")
                line = line[assign + 2 :]
                semi = line.find(";")
                num = line[:semi]
                if num.startswith("0x"):
                    num = hex_n(num)
                else:
                    num = int(num)
                data["size"] = num
                if derived_size:
                    size = max(size, num)
        if "offset" in data:
            if "size" not in data:
                line = decompiled.split("\n")[1]
                if "[2]" in line:
                    assign = line.find("=")
                    line = line[assign + 2 :]
                    semi = line.find(";")
                    num = line[:semi]
                    if num.startswith("0x"):
                        num = hex_n(num)
                    else:
                        num = int(num)
                    data["size"] = num
                    if derived_size:
                        size = max(size, num)
                else:
                    data["size"] = 1
                    if derived_size:
                        size = max(size, 1)
            things.append(data)
    fields = content[name]
    for thing in things:
        thing["type"] = fields[thing["field"]][0]
        thing["comment"] = fields[thing["field"]][1]
    return things, size


def create_type(dtm, name, size):
    existing = dtm.getDataType("noita.exe/" + name)
    if existing is None:
        category = CategoryPath("/noita.exe")
        struct = StructureDataType(category, name, size)
        struct = dtm.addDataType(struct, DataTypeConflictHandler.REPLACE_HANDLER)
    else:
        struct = existing
    return struct

def construct_structs(defs, name, size):
    data_type_manager = program.getDataTypeManager()
    # NOTE: hax here
    print(name, size)

    struct = StructureDataType(name, size)
    struct.replaceAtOffset(
        0,
        create_type(data_type_manager, "Component", 0x48),
        0x48,
        "inherited_fields",
        "",
    )
    defs.sort(lambda x, y: x["offset"] > y["offset"])
    for thing in defs:
        ptr = False
        if thing["type"][-1] == "*":
            ptr = True
            thing["type"] = thing["type"][:-1]
        ty = data_type_manager.getDataType("/" + thing["type"])
        if ty is None:
            ty = create_type(data_type_manager, thing["type"], thing["size"])
        if ty is None:
            print("cant find: " + thing["type"] + " " + str(thing["size"]))
            ty = ArrayDataType(
                data_type_manager.getDataType("/undefined1"), thing["size"], 1
            )
        if ptr:
            ty = data_type_manager.getPointer(ty)
        struct.replaceAtOffset(
            thing["offset"], ty, thing["size"], thing["field"], thing["comment"]
        )
    data_type_manager.addDataType(struct, DataTypeConflictHandler.REPLACE_HANDLER)

    # ty = data_type_manager.getDataType("/" + )
    # parser = CParser(data_type_manager)
    # parsed_datatype = parser.parse(struct_str)
    # data_type_manager.addDataType(
    #     parsed_datatype, DataTypeConflictHandler.DEFAULT_HANDLER
    # )


def get_all():
    table = program.getSymbolTable()
    addrs = []
    for i in table.getClassNamespaces():
        search = "Component"
        n = i.name
        if n[-len(search) :] != search or n == search:
            continue
        for s in table.getChildren(i.symbol):
            if s.name != "vftable":
                continue
            print(n)
            print(s.address)
            # NOTE: hax here
            addrs.append((s.address, n))
    return addrs


content = get_types_file()


def do(pair):
    if pair[1] in content:
        things, size = do_vftable(pair[0], content, pair[1])
        construct_structs(things, pair[1], size)


# do_vftable(currentAddress, content, "AIAttackComponent")
[do(x) for x in get_all()]
