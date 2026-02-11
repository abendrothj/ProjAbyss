# Underwater Exploration – Depth, Swim vs Sub, Implementation

This document describes how players explore the ocean: when they can swim, when the sub is needed, and how this is implemented in code.

---

## 1. Design Intent: Sub Only for Excessively Deep Parts

The submersible is **not** required for most underwater exploration. Players can:

- **Swim freely** in shallows (0–20m) and mid-depth (20–50m)
- **Use the sub optionally** for convenience (faster travel, oxygen refill) in those zones
- **Require the sub** only in deep (50–80m) and abyss (80m+) zones, where pressure and darkness make swimming impractical

This keeps the ocean feel open and explorable. The sub is a tool for reaching the bottom, not a gate for the whole game.

---

## 2. Depth Zones (Reference)

| Zone | Depth (m) | Y range (approx) | Swim | Sub |
|------|-----------|------------------|------|-----|
| Surface | 0 | y ≥ SEA_LEVEL | Walk | – |
| Shallows | 0–20 | -2 to -22 | ✓ | Optional |
| Mid | 20–50 | -22 to -52 | ✓ | Optional |
| Deep | 50–80 | -52 to -80 | Limited | ✓ Recommended |
| Abyss | 80+ | &lt; -80 | No | ✓ Required |

*Depth = SEA_LEVEL - y. SEA_LEVEL = -2, seafloor = -80. Map: 5km × 5km horizontal (MAP_SIZE).*

---

## 3. Current Implementation

### 3.1 Swimming

- **character.rs:** Walk when `pos.y >= wave_height + 0.6` (above surface). Swim when below.
- **Physics:** No gravity when swimming. 6DOF (WASD + Space/Shift). Water drag.
- **Oxygen:** Character has oxygen (60 s max, 1.2/s drain when swimming). Refills at surface (25/s). Respawn at Safe Island on drown.
- **Pressure:** Beyond 50 m depth, oxygen drains 3× faster. Enforces "sub required for deep."

### 3.2 Submersible

- **diving_bell.rs:** Sub has Oxygen when below `wave_height_at(pos)`.
- **Oxygen UI:** Bar shown when in sub. Drain rate 2.0/sec when underwater.
- **Movement:** Neutral buoyancy, 6DOF. Drive, ascend, descend.

### 3.3 Winch

- **winch.rs:** RopeJoint tethers sub to ship. Max length 100m. R/T to reel in/out when in boat.
- **Visual:** Cable mesh between ship and sub anchors.

### 3.4 Thresholds

| Use | Threshold | Source |
|-----|-----------|--------|
| Swim vs walk | `pos.y < wave_height + 0.6` | character.rs |
| Character oxygen drain | `pos.y < wave_height + 0.6` | character.rs |
| Pressure zone (>3× drain) | `depth > 50 m` | character.rs |
| Sub oxygen drain | `pos.y < wave_height_at(pos)` | diving_bell.rs |
| Marine snow | `cam.y < wave_height - 0.3` | marine_snow.rs |
| Depth color/fog | `depth = SEA_LEVEL - y > 0` | player.rs |

---

## 4. Pressure & Depth Limits (Implemented)

Pressure enforces "sub only for excessively deep parts":

1. **Pressure mechanic:** Beyond 50 m depth, oxygen drains 3× faster when swimming (~20 s to drown).
2. **Oxygen for character:** 60 s max, 1.2/s drain (3× at 50 m+). Refills at surface.
3. **Zone-based content:** Future – spawn artifacts, caves, biomes by depth.

---

## 5. Content Placement by Zone

| Zone | Current | Target |
|------|---------|--------|
| Surface | Islands, ship, buoys | Same |
| Shallows | Rocks, seaweed, buoys | Reefs, kelp, small fish |
| Mid | Debris, artifacts (3) | Wrecks, debris, schools |
| Deep | – | Caves, ruins, heavy artifacts |
| Abyss | Seafloor | Rift entrances, end-game loot |

---

## 6. Summary

- **Swim:** 0–50 m (oxygen limited; pressure zone at 50 m+ drains 3× faster).
- **Sub optional:** 0–50 m (convenience).
- **Sub required:** 50 m+ (pressured; swimming very risky).
- **Design:** Underwater exploration first; sub only for excessively deep parts.
