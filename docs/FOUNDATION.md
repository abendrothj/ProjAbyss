# ProjAbyss – Foundation Systems

Documentation of all foundational systems implemented as of 2026-02-10.

---

## 1. Game States

| State | Behavior |
|-------|----------|
| **Menu** | Full-screen overlay. "PROJECT ABYSS" + "Press Enter or E to Start". Cursor released. |
| **Playing** | All game systems run. Escape → Pause. |
| **Paused** | Overlay "PAUSED - Press Escape to Resume". Cursor released. Game systems gated off. |

**Module:** `game_state.rs`  
**Plugin:** `GameStatePlugin`  
**Systems:** `OnEnter`/`OnExit` for overlays, `menu_input`, `pause_input`, `handle_escape_pause`

---

## 2. Character Oxygen & Death

| Feature | Value |
|---------|-------|
| Max oxygen | 60 s |
| Drain rate (surface) | 1.2/s when swimming |
| Refill rate | 25/s at surface |
| Pressure zone | > 50 m depth: 3× drain |

**Death:** Oxygen = 0 → instant respawn at Safe Island (ship deck), oxygen reset, velocity cleared.

**Module:** `character.rs`  
**Components:** `CharacterOxygen`  
**UI:** Oxygen bar (bottom-left, 54px from bottom) when swimming. Same style as sub oxygen bar.

---

## 3. Pressure / Depth Limits

| Depth | Effect |
|-------|--------|
| 0–50 m | Normal oxygen drain (1.2/s) |
| > 50 m | 3× oxygen drain (~20 s to drown) |

Enforces "sub required for deep" per DESIGN.md. No hard block; swimming possible but very risky.

**Module:** `character.rs`  
**Constants:** `PRESSURE_DEPTH_THRESHOLD` (50 m), `PRESSURE_DRAIN_MULTIPLIER` (3.0)

---

## 4. Generic Interaction System

| Component | Purpose |
|-----------|---------|
| `Interactable` | Marks entity as interactable. `kind`, `range`. |
| `InteractKind` | EnterShip, EnterSubmersible, Pickup { item_id }, AttachToWinch { item_id } |

**Helpers:**
- `nearest_interactable_in_range(pos, query)` → `Option<(Entity, &InteractKind, dist_sq)>`
- `nearest_interactable_out_of_range(pos, interact_range, hint_range, query)` → `Option<dist_sq>` (for "move closer" prompt)

**Extending:** Add variant to `InteractKind`, implement prompt(), add match arm in `toggle_boat_enter` (player.rs).

**Module:** `interaction.rs`  
**Used by:** Ship, Sub, Artifacts (via Interactable component)

---

## 5. Vehicle UX Prompts

| Context | Prompt |
|---------|--------|
| In sub, artifact attached | "Press E to detach from winch" |
| In sub, near heavy artifact | "Press E to attach X to winch" |
| In vehicle (boat or sub, no attach) | "Press E to exit vehicle" |
| On foot, in range (≤6 m) | "Press E to enter ship" / "Press E to enter submersible" / "Press E to pick up X" (nearest) |
| On foot, 6–15 m | "Move closer to enter (6m)" |
| > 15 m | Hidden |

**Module:** `player.rs`  
**Constant:** `VEHICLE_ENTER_RANGE` = 6.0 m  
**UI:** Centered panel 120px from bottom, dark background, FiraSans-Bold

---

## 6. Save / Load

| Key | Action |
|-----|--------|
| F5 | Save |
| F9 | Load |

**File:** `save.ron` (workspace root)  
**Format:** RON (serde)  
**Saved:** Ship, sub, character positions + velocities; PlayerMode; WinchState cable_length  

**Load behavior:** Restores positions; always puts player on-foot (camera on character). Vehicle mode not restored (simplification).

**Module:** `save_load.rs`  
**Plugin:** `SaveLoadPlugin`  
**Dependencies:** ron, serde

---

## 7. Artifacts & Inventory

| Feature | Implementation |
|---------|----------------|
| Light artifacts | 3 cuboids on seafloor. `InteractKind::Pickup`. E in range → add to Inventory, despawn. |
| Heavy artifacts | 2 cuboids (1×1×1.2) at depth. `InteractKind::AttachToWinch`. E in sub → attach to winch. Reel in (R) to deliver. |
| Attach | In sub, near heavy artifact, E attaches as child of sub (hangs 4 m below). Follows sub. |
| Detach | In sub with artifact attached, E drops at current position. Restores physics and Interactable. |
| Delivery | Cable at min length (5 m) with artifact attached → add to inventory, despawn, `ArtifactPickupEvent`. |
| Inventory | `Resource` with `Vec<String>`. Inventory UI when items exist. |

**Module:** `artifacts.rs`  
**Plugin:** `ArtifactsPlugin`  
**Components:** `Artifact`, `Interactable` (Pickup or AttachToWinch)  
**Resources:** `Inventory`, `AttachedArtifact` (Option&lt;Entity&gt;)

**Note:** Save/load does not persist inventory. Future: add to SaveData.

---

## 8. Audio

| Event | Action |
|-------|--------|
| Artifact pickup | Play `assets/audio/pickup.ogg` |

**Module:** `audio.rs`  
**Plugin:** `AudioPlugin`  
**Message:** `ArtifactPickupEvent` (Bevy 0.17 Message API)  
**Setup:** Add `assets/audio/pickup.ogg` (ogg, wav, or mp3) to enable.

---

## 9. Module Index

| Module | Purpose |
|--------|---------|
| `main.rs` | App setup, plugins, setup_scene |
| `game_state.rs` | Menu, pause, states |
| `interaction.rs` | Interactable, InteractKind, helpers |
| `ocean.rs` | Gerstner waves, water mesh, OceanSolver |
| `ship.rs` | Dynamic ship, buoyancy, engine |
| `diving_bell.rs` | Submersible, oxygen, headlight |
| `winch.rs` | RopeJoint, cable visual, R/T reel in/out, deliver_attached_artifact when cable at min |
| `world.rs` | MAP_SIZE, MAP_FLOOR_Y, spawn position |
| `character.rs` | First-person, swim, oxygen, respawn |
| `player.rs` | Mode switch, camera, prompts, depth color/fog |
| `islands.rs` | Organic blobs, compound shapes |
| `scatter.rs` | Rocks, seaweed, debris, buoys |
| `marine_snow.rs` | Underwater particles |
| `save_load.rs` | F5/F9 persistence |
| `artifacts.rs` | Artifacts, inventory, inventory UI |
| `audio.rs` | Pickup sound |
| `fauna.rs` | Boids (schooling fish), flee from player/sub |
| `settings.rs` | InputBindings, GameSettings |

---

## 10. Remaining Gaps (Scan)

### High Priority (Core Loop)

| Area | Status | Notes |
|------|--------|------|
| **Inventory UI** | Done | Bottom-right panel when items; lists count + names |
| **Save inventory** | Done | SaveData.inventory_items; restored on load |
| **Heavy artifacts** | Done | Attach from sub, reel in (R) to deliver. Detach (E) to drop. |

### Medium Priority (Polish)

| Area | Status | Notes |
|------|--------|------|
| **Input mapping** | Done | InputBindings resource; all systems use it |
| **Settings** | Done | GameSettings.mouse_sensitivity |
| **Pickup sound** | Optional | Add assets/audio/pickup.ogg to enable |

### Content (VISION Phases)

| Area | Phase | Notes |
|------|-------|------|
| Boids/Fauna | 2 | Done – 5 schools, cohesion/separation/alignment, flee |
| Shark | 2 | Patrol, chase |
| Wind/Storms | 3 | Ship speed, visual |
| Ship damage | 3 | Hull health, repair |
| Crafting | 4 | O₂ tank, flippers |
| Voyages | 4 | Quest: "Find artifact at X" |
| Biome zones | 5 | Depth-based scatter |
| Wrecks | 5 | Partial hulls, loot |
| Day/night | 6 | Lighting, bioluminescence |
| More fauna | 6 | Stingrays, eels |
| Lore | 6 | Data logs in wrecks |
| Networking | 6 | Co-op |

### Intentional

| Area | Notes |
|------|-------|
| Water collision | Player falls through to seafloor; no water mesh collider |

---

## References

- **activity.log** – Changelog with dates
- **docs/DESIGN.md** – Vision, depth zones
- **docs/EXPLORATION.md** – Depth thresholds (update per pressure impl)
- **docs/VISION_SUBNAUTICA_SOT.md** – Phase roadmap
