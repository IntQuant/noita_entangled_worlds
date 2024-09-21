local loadentity = EntityLoad
function EntityLoad(filename, x, y)
    if filename ~= "data/entities/animals/boss_centipede/ending/gold_effect.xml" then
        loadentity(filename, x, y)
    end
end