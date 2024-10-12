local module = {}

function module.on_world_update()
    if GameGetFrameNum() % 60 == 0 then
        GamePrint("Hi from api example!")
    end
end

return module
