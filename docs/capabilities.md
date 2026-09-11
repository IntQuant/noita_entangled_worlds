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
 - `on_poly_death()`

Provided by:
 - damage (shared health) system - provides all of the functions above
 - local health system - provides only `do_game_over` and `on_poly_death`; the
   other five are commented out, so they are `nil` in `local_health` game mode

Used by:
 - heart pickups system - `health`, `max_health`, `set_health`, `set_max_health`
   (loaded only in `shared_health` game mode)
 - patch meat biome system - `health`, `set_health`
   (loaded only in `shared_health` game mode)
 - perk patches system - `health`, `max_health`, `set_health`, `set_max_health`
 - polymorph system - `on_poly_death`
 - local health system - `on_poly_death`
