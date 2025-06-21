local function string_split(s, splitter)
    local words = {}
    if s == nil or splitter == nil or s == "" then
        return words
    end
    for word in string.gmatch(s, "([^" .. splitter .. "]+)") do
        table.insert(words, word)
    end
    return words
end
local function contains(table, element)
    for _, value in pairs(table) do
        if value == element then
            return true
        end
    end
end
local t = string_split(CrossCall("ew_banned_spells"), ",")
local old = GetRandomActionWithType
function GetRandomActionWithType(...)
    local n = old(...)
    local a = { ... }
    while contains(t, n) do
        a[1] = a[1] + 1
        a[2] = a[2] + 1
        n = old(unpack(a))
    end
    return n
end
local old2 = GetRandomAction
function GetRandomAction(...)
    local n = old2(...)
    local a = { ... }
    while contains(t, n) do
        a[1] = a[1] + 1
        a[2] = a[2] + 1
        n = old2(unpack(a))
    end
    return n
end
