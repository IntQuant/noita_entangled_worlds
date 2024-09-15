ew_orig_generate_shop_item = generate_shop_item
ew_orig_generate_shop_wand = generate_shop_wand

function generate_shop_item( x, y, cheap_item, biomeid_, is_stealable )
    CrossCall("ew_sync_gen", "generate_shop_item", x, y, cheap_item, biomeid_, is_stealable)
end

function generate_shop_wand( x, y, cheap_item, biomeid_, is_stealable )
    CrossCall("ew_sync_gen", "generate_shop_wand", x, y, cheap_item, biomeid_, is_stealable)
end