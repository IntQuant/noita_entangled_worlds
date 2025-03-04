local function string_split(s, splitter)
    local words = {}
    if s == nil or splitter == nil or s == "" then
        return {}
    end
    for word in string.gmatch(s, "([^" .. splitter .. "]+)") do
        table.insert(words, word)
    end
    return words
end
local t = string_split(CrossCall("ew_banned_spells"), ",")
local old = GetRandomActionWithType
function GetRandomActionWithType(...)
    local n = old
    while table.contains(t, n) do
        n = old
    end
    return n
end
old = GetRandomAction
function GetRandomAction(...)
    local n = old
    while table.contains(t, n) do
        n = old
    end
    return n
end
