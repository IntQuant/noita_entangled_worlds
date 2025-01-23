-- Parts of EW api that are guaranteed to be stable and can be used from any lua context.

local api = {}

-- Resends current inventory, in case a mod caused it to change in a non-vanilla way (like editing properties of an item).
function api.force_update_inventory()
    CrossCall("ew_api_force_send_inventory")
end

return api
