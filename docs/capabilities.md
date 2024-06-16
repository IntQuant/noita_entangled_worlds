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

Provided by:
 - damage (shared health) system

Used by:
 - heart pickups (core)
