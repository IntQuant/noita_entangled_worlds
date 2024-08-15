--- add_new_genome taken from noita wiki.

function split_string(inputstr, sep)
    sep = sep or "%s"
    local t= {}
    for str in string.gmatch(inputstr, "([^"..sep.."]+)") do
        table.insert(t, str)
    end
    return t
end

local content = ModTextFileGetContent("data/genome_relations.csv")

--The function works like this: genome_name is the name of your new genome/faction,
--default_relation_ab is the relation with all the horizontal genomes which relations weren't specified in the table,
--default_relation_ba is the relation with all the vertical genomes which relations weren't specified in the table,
--self relation is the genome's relation with itself,
--relations is a table which directly specifies the value of the genome relation with.

local function add_new_genome(genome_name, default_relation_ab, default_relation_ba, self_relation, relations)
    local lines = split_string(content, "\r\n")
    local output = ""
    local genome_order = {}
    for i, line in ipairs(lines) do
        if i == 1 then
            output = output .. line .. "," .. genome_name .. "\r\n"
        else
            local herd = line:match("([%w_-]+),")
            output = output .. line .. ","..(relations[herd] or default_relation_ba).."\r\n"
            table.insert(genome_order, herd)
        end
    end

    local line = genome_name
    for _, v in ipairs(genome_order) do
        line = line .. "," .. (relations[v] or default_relation_ab)
    end
    output = output .. line .. "," .. self_relation

    return output
end

content = add_new_genome("notplayer", 100, 100, 100, {
    player = 0,
})

content = add_new_genome("player_pvp", 0, 0, 0, {})

ModTextFileSetContent("data/genome_relations.csv", content)

return {}