# Capabilities

Capabilities allow several systems provide same functionality in different ways. Exactly one capability implementation should be active.

Capabilities are registered by having an entry in `ctx.cap` table.

# List of capabilities

## `health` capability

Functions:
 - `health() -> num`
 - `max_health() -> num`
 - `set_health(hp: num)`
 - `set_max_health(hp: num)`
 - `inflict_damage(dmg: num)`
 - `do_game_over(msg: str)`
 - `on_poly_death(msg: str)`

Provided by:
 - damage (shared health) system
 - local health system

Used by:
 - heart pickups system

## 'item_sync' capability

Functions:
 - `globalize(entity_id, instantly: bool)`
 - `register_pickup_handler(fn(local_item_id))`

Provided by:
 - item_sync system
