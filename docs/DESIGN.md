# Project Abyss – Design Document

**Vision:** An underwater exploration game where players sail, swim, and dive to discover the ocean. The submersible is a tool for *excessively deep* exploration—not a requirement for enjoying the underwater world.

---

## 1. Core Pillar: Underwater Exploration

Project Abyss is first and foremost an **underwater exploration game**. The ocean is the main space of play: swim through kelp forests, skirt island shores, discover wrecks and reefs, and descend into the abyss when you're ready.

### What Makes It Work

- **Freedom to explore on foot (or fins):** Jump off the ship and swim. No forced sub usage for shallow or mid-depth areas.
- **Depth as progression:** Early game = islands, shallows, reefs. Late game = deep rift caves, seafloor ruins, heavy artifacts.
- **Atmosphere over urgency:** The ocean should feel vast, mysterious, and sometimes frightening—not like a checklist of tasks. Discovery is the reward.
- **Co-op synergy:** One player can stay on the ship (winch, navigation) while the other explores. Both roles matter.

---

## 2. Depth Zones & When to Use the Sub

The ocean is layered by depth. Each zone has different affordances and risks.

| Zone | Depth (m) | Swim? | Sub? | Notes |
|------|-----------|-------|------|-------|
| **Surface** | 0 | No (walk on islands/ship) | No | Sailing, islands, buoys |
| **Shallows** | 0–20 | ✓ Yes | Optional | Bright, safe, reefs, kelp. Swim freely. |
| **Mid-Depth** | 20–50 | ✓ Yes | Optional | Murky, oxygen matters. Sub speeds travel. |
| **Deep** | 50–80 | Limited | ✓ Recommended | Dark, pressure risk. Sub provides O₂, light. |
| **Abyss** | 80+ | No | ✓ Required | Pitch black, heavy artifacts. Sub only. |

### Design Principle: Sub for Excessively Deep Parts

- **Shallows & Mid:** Players can and should explore by swimming. The sub is available for convenience (faster travel, oxygen refill) but not mandatory.
- **Deep & Abyss:** Pressure, darkness, and oxygen limits make swimming impractical or fatal. The sub becomes the safe zone and the only way to reach the bottom.

This keeps the sub special: it unlocks the deepest, most rewarding content instead of gatekeeping the whole ocean.

---

## 3. Exploration Loop (Not Just Extraction)

The loop is **Sail → Explore → Dive → Extract → Survive**, with exploration as the core activity.

1. **Sail** – Move the ship to points of interest. Dynamic waves, storms, navigation.
2. **Explore** – Swim around islands, reefs, wrecks. Discover caves, collectibles, lore. No sub required for most of this.
3. **Dive** – When ready, deploy the sub. Use the winch to lower it. One player operates the winch; the other pilots.
4. **Extract** – Find heavy artifacts in the deep. Hook them to the winch. Reel up with the loot.
5. **Survive** – Oxygen, storms, predators. Return to port with the loot.

Exploration is the glue: players discover *where* to dive and *what* to extract by exploring first.

---

## 4. Two-Layer Physics & Movement

### Layer 1: Surface (Above Water)

- **Ship:** Dynamic rigid body, buoyancy, engine. Rocks and tilts with waves.
- **Character:** Walk/jump on islands and ship deck. Gravity, collision.
- **Islands:** Safe zones, supplies, spawn points.

### Layer 2: Underwater (Below Water)

- **Swimming:** 6DOF movement. No gravity. Space = ascend, Shift = descend. Limited oxygen when underwater.
- **Submersible:** 6DOF, neutral buoyancy. Holds depth. Oxygen provided. Required for deep/abyss.
- **Winch:** Cable tethers sub to ship. Reel in/out. Ship reacts to tension when cable is taut.

---

## 5. Content by Zone (Target State)

| Zone | Content | Swim | Sub |
|------|---------|------|-----|
| **Surface** | Islands, ship, buoys | Walk | – |
| **Shallows** | Reefs, kelp, rocks, small fish | ✓ | Optional |
| **Mid** | Wrecks, scattered debris, schools | ✓ | Optional |
| **Deep** | Caves, ruins, heavy artifacts | Limited | ✓ |
| **Abyss** | Rift entrances, end-game loot | No | ✓ |

---

## 6. Map Scale (Big Map)

| Constant | Value | Purpose |
|----------|-------|---------|
| `MAP_SIZE` | 5000.0 | Ocean & seafloor extent (5km × 5km) |
| `MAP_FLOOR_Y` | -80.0 | Seafloor depth |
| `SEA_LEVEL` | -2.0 | Mean water surface |
| Swim threshold | `wave_height + 0.6` | Exit water when above this |
| Oxygen drain | `wave_height_at(pos)` | Sub oxygen when below surface |

The world is 5km × 5km for a large, explorable ocean. Islands and scatter are spread across the map.

---

## 7. References

- **proj.md** – Original design bible (UE5 legacy, extraction loop)
- **docs/EXPLORATION.md** – Depth thresholds, swim vs sub, implementation notes
- **docs/GRAPHICS_UPGRADE_ROADMAP.md** – Visual direction (biomes, depth fade)
- **activity.log** – Implementation changelog
