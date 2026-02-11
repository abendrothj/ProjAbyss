# Project Abyss – Subnautica + Sea of Thieves Vision

How to make Project Abyss a true fusion: **Subnautica** depth, wonder, and thalassophobia meets **Sea of Thieves** sailing, co-op, and emergent play—with a **living** underwater world.

---

## 1. What Each Game Brings

### Subnautica DNA

| Element | What It Does |
|---------|--------------|
| **Biomes** | Distinct zones: Shallows, Kelp Forest, Grassy Plateaus, Crash Zone, Dunes, Lost River, Lava Zones. Each has color, flora, fauna, landmarks. |
| **Fauna** | Schools of fish, passive grazers, ambush predators, leviathans. AI: boids, flee, chase, idle. |
| **Flora** | Coral, kelp, creepvine, mushrooms. Harvestable, some dangerous. |
| **Atmosphere** | Light fades by depth. Red gone at 10m, blue at 100m. Bioluminescence in the dark. Thalassophobia. |
| **Discovery** | Wrecks, abandoned bases, data logs. Story through exploration. |
| **Survival** | Oxygen, food, water. Crafting. Base building (optional). |
| **Vehicles** | Seamoth (shallow), Cyclops (deep), Prawn (walking). Each unlocks new zones. |

### Sea of Thieves DNA

| Element | What It Does |
|---------|--------------|
| **Sailing** | Physics-based. Wind, waves, sails, steering. Ship as home. |
| **Co-op** | Crew roles: helm, sails, lookout, bilge. Everyone matters. |
| **Emergent play** | Storms, other crews, skeleton ships, kraken. Unscripted moments. |
| **Voyages** | Quests from islands. Dig, deliver, fight. |
| **Loot** | Treasure chests, skulls, artifacts. Turn in at outposts. |
| **Shared world** | Other players. PvP optional. Server as stage. |

### The Fusion

| Subnautica | Sea of Thieves | Project Abyss |
|------------|----------------|---------------|
| Biomes | Sailing | Depth-based biomes + surface islands |
| Fauna | Ship as home | Living fish + ship as base |
| Discovery | Loot | Wrecks + artifacts + winch extraction |
| Survival | Voyages | O₂, pressure + sail-to-dive loop |
| Thalassophobia | Emergent | Storms, sharks, darkness |

---

## 2. Living Underwater World

### 2.1 Fauna – Fish & Creatures

**Passive (schools, grazers):**
- **Small fish** – Boids. School together, flee from player. 10–30 cm. Multitude.
- **Medium fish** – Solo or small groups. Reef fish, parrotfish. 30–80 cm.
- **Large passive** – Stingrays, groupers. 1–2 m. Slow, curious.

**Predators:**
- **Small** – Eels, barracuda. Ambush from rocks. 0.5–1 m.
- **Medium** – Sharks. Patrol, chase when close. 2–3 m. Threat in mid-depth.
- **Large** – Leviathan-class (optional). Rare, deep only. 10+ m. Avoid or flee.

**AI behavior:**
- **Boids** – Cohesion, separation, alignment. Move in schools.
- **Flee** – Run from player or predator when within threshold.
- **Chase** – Predator pursues prey or player when in range.
- **Idle** – Swim in circles, hover near rocks.

**Implementation path:**
1. Boids (passive schools) – no navmesh, simple steering.
2. Flee behavior – repulsion from player.
3. Shark (one type) – chase when player in range, patrol otherwise.
4. Leviathan (later) – scripted zones, roar, chase.

---

### 2.2 Flora – Plants & Coral

**Harvestable:**
- **Kelp** – Already have seaweed. Add harvest: "Kelp fiber" for crafting.
- **Coral** – Small clusters. Harvest: "Coral sample."
- **Creepvine** – Tall stalks in kelp forest. Harvest: "Creepvine seed."

**Decorative:**
- **Anemones** – Clusters on rocks. Can sting (minor damage).
- **Sea grass** – Patches on sand. Sways with current.
- **Bioluminescent** – Glowing bulbs in deep zones.

**Implementation path:**
1. More seaweed variants (height, color) by biome.
2. Coral clusters as scatter.
3. Harvest interaction (E near plant → add to inventory).
4. Bioluminescent props in deep zones.

---

### 2.3 Biomes – Depth + Terrain

| Biome | Depth | Terrain | Flora | Fauna | Mood |
|-------|-------|---------|-------|-------|------|
| **Surface** | 0 | Islands, buoys | – | Gulls (optional) | Bright, safe |
| **Shallows** | 0–20 m | Sand, reefs | Kelp, coral | Schools, parrotfish | Colorful |
| **Kelp Forest** | 20–50 m | Tall kelp | Creepvine | Schools, eels | Murky green |
| **Grassy Plateaus** | 20–50 m | Flat, grass | Sea grass | Stingrays, reef fish | Open |
| **Wreck Zone** | 30–60 m | Debris, hulls | Barnacles | Barracuda, sharks | Eerie |
| **Midnight Zone** | 50–80 m | Rocky, dark | Biolum | Rare fish, sharks | Pitch black |
| **Abyss** | 80+ m | Rift, caves | Glow flora | Leviathan (optional) | Horror |

**Implementation path:**
1. Depth-based spawn: scatter and fauna by `y` (depth).
2. Terrain variation: noise-based "biome" per region (XZ + depth).
3. Color grading per biome: green (kelp), blue (plateaus), black (midnight).
4. Fog density per depth.

---

## 3. Sea of Thieves Layer – Ship & Sailing

### 3.1 Ship as Home

- **Crew roles:** Helm (steer), Winch (reel), Lookout (scan). All useful.
- **Ship storage:** Crate for artifacts. Oxygen tanks. Repair wood.
- **Cooking (optional):** Raw fish → cooked. Hunger meter.
- **Repairs:** Hull damage from storms/collision. Repair with hammer + wood.

### 3.2 Sailing & Weather

- **Wind:** Direction and strength affect speed. Sail trim (optional).
- **Storms:** Heavy rain, waves, lightning. Ship takes damage. Navigate or anchor.
- **Day/night:** Affects visibility underwater. Bioluminescence at night.

### 3.3 Voyages & Quests

- **Maps:** Buy/find map with marked dive site.
- **Rumors:** NPC at Safe Island: "Wreck seen near X."
- **Deliveries:** Ferry artifact from A to B. Time limit?

---

## 4. Subnautica Layer – Survival & Progression

### 4.1 Oxygen

- **Character:** O₂ meter when swimming. Surface or sub to refill.
- **Oxygen tank:** Craftable upgrade. Extends swim time.
- **Sub:** O₂ provided. Drain when underwater. Refill at surface.

### 4.2 Pressure (Optional)

- **Depth limit:** Beyond ~50 m without sub = damage or death.
- **Sub:** Pressure hull. Safe to 80 m+.

### 4.3 Crafting & Resources

- **Gather:** Kelp, coral, metal (wreck scrap), sulfur (thermal vents).
- **Craft:** O₂ tank, flippers (swim speed), repair kit, flashlight.
- **Inventory:** Simple slots. Hold 5–10 items.

### 4.4 Base Building (Optional, Late)

- **Underwater base:** Place foundation, walls, hatches. O₂ generator.
- **Or:** Ship only. No base. Simpler.

---

## 5. Content Flow – What to Build First

### Phase 1: Foundation (Done)

- [x] Ship, sub, character, winch
- [x] Ocean, islands, scatter
- [x] Swim, depth color, marine snow
- [x] Big map (5 km)

### Phase 2: Living World (Next)

- [ ] **Boids** – Schooling fish. One species, 20–50 fish. Passive.
- [ ] **Biome scatter** – Depth-based: more kelp in shallows, coral in reefs.
- [ ] **Shark** – One type. Patrol, chase when close. 2–3 m.
- [x] **Character O₂** – Oxygen meter when swimming. Surface to refill. Respawn on drown.

### Phase 3: Sailing & Weather

- [ ] **Wind** – Affects ship speed. Simple direction + strength.
- [ ] **Storms** – Occasional. Big waves, rain. Visual.
- [ ] **Ship damage** – Hull health. Repair with wood.

### Phase 4: Extraction & Progression

- [x] **Artifacts** – Pickable objects (3 on seafloor). Pick up, add to inventory.
- [x] **Inventory** – Basic (Vec<String>). No UI yet. Expand: attach heavy artifacts to winch.
- [ ] **Crafting** – O₂ tank, flippers. From kelp, scrap.
- [ ] **Voyages** – Simple quest: "Find artifact at X."

### Phase 5: Depth & Biomes

- [x] **Pressure** – 3× oxygen drain below 50 m without sub.
- [ ] **Biome zones** – Kelp forest, wreck zone, midnight. Distinct look.
- [ ] **Wrecks** – Partial hulls. Loot inside. Barracuda.
- [ ] **Leviathan (optional)** – Deep only. Roar, chase. Avoid.

### Phase 6: Polish & Emergence

- [ ] **Day/night** – Lighting, bioluminescence.
- [ ] **More fauna** – Stingrays, eels, schools.
- [ ] **Lore** – Data logs in wrecks. Story through discovery.
- [ ] **Co-op tuning** – Shared inventory? Roles?

---

## 6. Technical Notes (Bevy)

### Fauna

- **Boids:** `RigidBody::KinematicVelocityBased` or custom velocity. Steering: cohesion, separation, alignment. No navmesh.
- **Shark:** Same. Chase = move toward player when in range. `Transform` + `Velocity`.
- **Spawning:** By biome. Query region (XZ + depth), spawn N fish of type T.

### Biomes

- **Depth:** `y < threshold` → biome.
- **Region:** Hash of (x/500, z/500) → biome seed. Consistent per region.
- **Scatter:** `spawn_scatter` checks biome, picks mesh set.

### Audio (Future)

- Ambient: surface (waves, wind), underwater (bubbles, hum).
- Fauna: fish schools (soft), shark (growl), Leviathan (roar).
- Ship: creak, sail flap, engine.

---

## 7. Summary – True Fusion Checklist

| Subnautica | Sea of Thieves | Project Abyss |
|------------|----------------|---------------|
| ✓ Biomes by depth | ✓ Ship as home | Depth biomes + ship base |
| ✓ Living fish | ✓ Sailing physics | Boids + sharks + ship |
| ✓ Oxygen, pressure | ✓ Storms | O₂ + storms + damage |
| ✓ Discovery (wrecks) | ✓ Loot, voyages | Artifacts + winch + quests |
| ✓ Thalassophobia | ✓ Co-op | Dark zones + crew roles |
| ✓ Crafting | ✓ Emergent | Resources + repair |

**Core loop:** Sail → Explore (swim/schools) → Dive (sub) → Extract (winch) → Survive (O₂, storms, sharks) → Return.

**Mood:** Wonder in the shallows. Tension in the deep. Ship as sanctuary. Ocean as character.
