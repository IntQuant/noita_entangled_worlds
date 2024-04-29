local first = true

local net = {
    get_events = function()
        if first then
            first = false
            return {
                { kind = "connect", peer_id = 1}
            }
        end
        return {}
    end
}

return net