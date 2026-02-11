# Scale Audit – Objects & Player

Reference: **1 human ≈ 1.7 m**. Map: **5 km × 5 km**.

---

## Player (character.rs)

| Property | Value | Notes |
|----------|-------|------|
| Mesh | Capsule 0.4 radius, 0.9 length | Total height ~1.7 m ✓ |
| Collider | capsule_y(0.45, 0.4) | ~1.7 m tall ✓ |
| Walk speed | 4.0 m/s | Jog / fast walk ✓ |
| Swim speed | 2.5 m/s | Plausible ✓ |
| Jump | 6.0 m/s | ~0.6 s to peak ✓ |
| Ascend/descend | 4.0 / 2.0 m/s | Swimming ✓ |
| Camera height | 0.9 m | Eye height ✓ |

**Verdict:** Player scale looks correct.

---

## Ship (ship.rs)

| Property | Value | Notes |
|----------|-------|------|
| Collider | cuboid(2.5, 0.7, 4.0) half-extents | 5 × 1.4 × 8 m total ✓ |
| Mesh scale | 2.5× | GLB scaled to match collider |
| Pontoon offsets | ±0.9, ±1.5 | In ship-local units |
| Engine power | 22000 | Arbitrary units |
| Buoyancy | 22000 per pontoon | Tuned for waves |

**Verdict:** Rowboat/small sailboat scale (~5–8 m). OK.

---

## Submersible (diving_bell.rs)

| Property | Value | Notes |
|----------|-------|------|
| Collider | cylinder(3.0, 2.5) | 6 m diameter, 5 m tall ✓ |
| Mesh scale | 4.0× | GLB may be authored small |
| Light range | 40 m | Headlight ✓ |
| Drive power | 15 | Velocity units |
| Ascend/descend | 8 m/s | Sub speed ✓ |

**Verdict:** Small submersible scale. OK.

---

## Islands (islands.rs)

| Island | Radius | Notes |
|--------|--------|-------|
| Safe Island | 14 m | Spawn point ✓ |
| Volcanic | 45–50 m | Large ✓ |
| Organic blobs | 22–35 m | Medium ✓ |
| Reef / atoll | 20–38 m | OK |

**Verdict:** Positions scaled by MAP_SCALE. Radii unchanged (world-space). OK.

---

## Scatter (scatter.rs)

| Object | Base size | Scale | Approx size |
|--------|-----------|-------|-------------|
| Rock boulder | Sphere 0.5 | 0.4–1.5 | 0.2–0.75 m |
| Rock pebble | 0.25×0.15×0.35 | 0.4–1.5 | Small |
| Seaweed | Capsule 0.06×0.7 | 0.6–1.2 | 0.04–0.8 m |
| Debris crate | 0.3×0.25×0.4 | 0.9×0.7×1.1 | ~0.3 m |
| Debris barrel | 0.15×0.25 | 1.0 | ~0.3 m diam |
| Buoys | GLB | 1.5× | Unknown |

**Post-audit changes (2026-02-10):**
- **Rocks:** Base 0.8 m, scale 0.8–2.0 → boulders ~1–2 m ✓
- **Debris:** Base 0.5×0.4×0.6 crate, 0.25×0.5 barrel, scale ~1.2 → ~0.8–1 m ✓
- **Seaweed:** 0.08×1.2 capsule, scale 0.8–1.6 → ~1–2 m ✓
- **Buoys:** Scale 1.5→2.0 for visibility ✓

---

## Winch cable (winch.rs)

| Property | Value | Notes |
|----------|-------|------|
| Cylinder | 0.08 radius, 1.0 height | 8 cm radius ✓ |
| Length | Dynamic (ship–sub) | OK |
| Anchors | SHIP_ANCHOR (0.6, 2.5), SUB_ANCHOR (0, 2.5) | Local space ✓ |

**Verdict:** Cable thickness OK.

---

## Marine snow (marine_snow.rs)

| Property | Value | Notes |
|----------|-------|------|
| Base mesh | 1×1×1 cube | Unit ✓ |
| Particle scale | 0.01–0.035 | 1–3.5 cm ✓ |
| Sphere radius | 10 m | Spawn radius ✓ |
| Recycle | 12 m | OK |

**Verdict:** Correct for floating debris.

---

## Interaction & physics

| Property | Value | Notes |
|----------|-------|------|
| VEHICLE_ENTER_RANGE | 6 m | E to enter ship/sub ✓ |
| Surface exit margin | 0.6 m | Swim vs walk threshold ✓ |
| Snap to ground | 0.2 m | Character controller ✓ |
| Winch max length | 100 m | Cable ✓ |

**Verdict:** All within human scale.

---

## Recommended adjustments

1. **Rocks:** Increase base or scale to reach ~1–2 m for boulders.
2. **Debris:** Scale crates/barrels from ~0.3 m to ~0.8–1.0 m.
3. **Seaweed:** Optional: increase to 1–2 m for kelp.
4. **Camera:** Eye height 0.9 m matches ~1.7 m character.

---

## Summary

| Category | Status |
|----------|--------|
| Player | ✓ OK |
| Ship | ✓ OK |
| Sub | ✓ OK |
| Islands | ✓ OK |
| Rocks | ✓ Adjusted (1–2 m boulders) |
| Debris | ✓ Adjusted (~1 m crates/barrels) |
| Seaweed | ✓ Adjusted (1–2 m) |
| Cable, marine snow, interaction | ✓ OK |
