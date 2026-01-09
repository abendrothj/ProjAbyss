# Project Abyss

A physics-driven ocean exploration prototype in Unreal Engine. Drive a ship over procedural waves and switch control between a marine character and the ship via a clean Interaction Interface.

## Overview
- Engine: Unreal Engine (Mac editor targets present)
- Languages: C++ for gameplay; Blueprints for assets and input bindings
- Key systems:
  - OceanSolver (procedural wave sampling)
  - MasterShip (buoyant pawn with enhanced input)
  - MarineCharacter (first-person character)
  - InteractableInterface (handshake for interacting with ships, loot, doors)

## Handshake (Character ↔ Ship)
1. Walk to the ship.
2. Press E.
3. Camera snaps to the ship.
4. You drive using W/A/S/D.

Implementation details:
- `InteractableInterface` defines `Interact(APawn* InstigatorPawn)` as a BlueprintNativeEvent.
- `AMasterShip` implements the interface and, on interact, unpossesses the instigator pawn, possesses the ship, applies the ship’s input mapping, and snaps view to its spring arm.
- `AMarineCharacter::Interact()` line-traces forward; if the hit actor implements the interface, it executes `Interact`.

## Controls
- Character
  - Move: WASD
  - Look: Mouse
  - Jump: Space
  - Interact: E
- Ship
  - Throttle: W/S
  - Turn: A/D

## Source Layout
- `Source/ProjAbyss/Public/InteractableInterface.h` – Interaction interface
- `Source/ProjAbyss/Public/MasterShip.h` and `Private/MasterShip.cpp` – Buoyancy, input, possession
- `Source/ProjAbyss/Public/MarineCharacter.h` and `Private/MarineCharacter.cpp` – Character, camera, interact raycast
- `Source/ProjAbyss/Public/OceanSolver.h` – Wave sampling utilities

## Setup (macOS)
1. Prereqs: Xcode, Unreal Engine (Mac), command line tools.
2. Clone repo.
3. Generate/refresh project files via Unreal (if needed).

## Build
- Xcode workspace is present; build the editor target:

```zsh
/Applications/Xcode.app/Contents/Developer/usr/bin/xcodebuild \
  -workspace "/Users/ja/Desktop/projects/ProjAbyss/ProjAbyss (Mac).xcworkspace" \
  -scheme "ProjAbyssEditor (Mac)" \
  -configuration Development
```

## Run
- Open the project in Unreal Editor.
- Play in editor.
- Walk to the ship hull and press E.
- Drive with W/A/S/D.

## Git Hygiene
- .gitignore excludes Unreal build/caches (`Binaries/`, `DerivedDataCache/`, `Intermediate/`, `Saved/`), shader debug/cache, per-user IDE files, and generated external actors/objects.
- Only core `Config/`, `Content/`, `Source/`, `.uproject`, and README are tracked.

## Plan (Derived from plan.pdf)
The following is a concise, actionable plan distilled from the project’s design intent and current implementation. For full details, consult `plan.pdf` in the repo.

### Goals
- Core interaction system via `InteractableInterface` to enable consistent player-object interactions (ships, loot, levers, doors).
- Seamless possession swap between Character and Ship with Enhanced Input contexts.
- Robust buoyancy and movement for the ship over dynamic waves.
- Clean content/project hygiene to keep builds and caches out of version control.

### Milestones
1. Handshake MVP (Done)
   - Interface created (`InteractableInterface`)
   - Ship implements `Interact(APawn*)` and handles possession swap
   - Character calls interface on E (raycast + `Execute_Interact`)
   - Camera switches to ship’s spring arm, driving enabled
2. Exit Interaction (Next)
   - Add E again or a separate key to exit ship
   - Re-possess character, restore mapping, snap view back to character camera
3. Interaction Targets
   - Add dedicated interaction components for the ship (e.g., wheel hotspot)
   - Expand to loot pickups, levers, doors via same interface
4. UX Polish
   - Interaction prompts (e.g., “Press E to Take Wheel”) when in range and looking at target
   - Smooth camera transition (blend) on possess/unpossess
5. Physics and Controls Tuning
   - Fine-tune float forces, drag, and torque for stability across wave conditions
   - Ship control deadzones and sensitivity, optional reverse/throttle ramp
6. Content Expansion
   - More sea states and environmental hazards
   - Additional ships and items implementing the interface

### Interaction Flow Contracts
- Input: E key (InteractAction) from `AMarineCharacter`
- Detection: 300 cm forward line trace from character camera to `ECC_Visibility`
- Dispatch: If hit actor implements `UInteractableInterface`, call `Execute_Interact(HitActor, CharacterPawn)`
- Ship Interact Behavior:
  - Unpossess instigator pawn’s controller
  - Possess ship (`AMasterShip`)
  - Apply ship’s Enhanced Input mapping context (priority > character)
  - Set view target to ship (SpringArm + FollowCamera)

Error modes and guards:
- If instigator lacks a controller, interaction is a no-op
- If mapping context is missing, ship may not respond to input—validate assignments
- Ensure collision is present on the ship hull so the trace can hit

### Exit Ship (Proposed Implementation)
- Keybind: E while driving (or another key like F)
- Logic:
  - Cache last character pawn on possess; store as `WeakObjectPtr<AMarineCharacter>`
  - On exit: current controller unpossesses ship, re-possesses cached character
  - Re-apply character mapping context (priority lower or distinct from ship)
  - Restore view target to character’s first-person camera

### Testing Checklist
- Handshake
  - Walk to ship, press E → possess ship, camera snaps, W/A/S/D drive
- Raycast
  - Aim at non-ship objects → no interaction; aim at hull → interaction fires
- Input contexts
  - Validate both character and ship have mapping contexts assigned; ship context priority higher when possessed
- Edge cases
  - Press E out of range → no-op
  - Possess without controller → safely ignored
  - Rapid E presses → idempotent behavior (consider cooldown if needed)

### Build and Run (macOS)
See Build and Run sections above. Ensure Xcode toolchain and Unreal version match project files.

### Repository Hygiene
- `.gitignore` excludes UE build/caches and per-user IDE artifacts
- ExternalActors/ExternalObjects are ignored due to volume and determinism; include only if team requires tracking

## Roadmap Snapshot
- Short term: Exit ship interaction, prompts, smoother camera blends
- Mid term: Interaction components, additional interactables (loot, levers, doors)
- Long term: Advanced sea states, multiple vessel types, save systems, content pipelines

## Troubleshooting
- If E doesn’t possess the ship, ensure the line trace hits the ship (aim at the hull collision) and the ship implements `InteractableInterface`.
- Verify Enhanced Input mapping contexts are assigned in the ship and character.
- If build fails, confirm Xcode toolchain and Unreal version match the project files.

## License
Proprietary project assets. Code intended for internal prototype use.

## Design Summary (from proj.md)
This section summarizes the game design and technical plan captured in `proj.md`.

### Executive Summary
- Title: Project Abyss (Working Title)
- Genre: Co-op Extraction Survival / Physics Sandbox
- Core Loop: Sail → Scan → Dive → Extract → Survive
- Hook: Sea of Thieves–style sailing physics meets Subnautica’s depth and horror, wrapped in an extraction shooter loop (without guns)
- Engine: Unreal Engine 5.7
- Target Platform: PC (Steam) first

### Gameplay Mechanics
- Extraction Loop
  - Preparation: Spawn on Safe Island, stock ship (fuel, oxygen tanks, repair wood)
  - Voyage: Physically sail to generated coordinates (dynamic waves, storms, navigation challenges)
  - Descent: Deploy Diving Bell from ship (pressure limits; Bell as safe zone)
  - Heist: Explore procedural “Rift” (Cave/Ruin) with O2 limits, darkness, predators; find heavy Artifacts
  - Retrieval: Artifacts too heavy to carry far—hook to Bell’s winch
  - Ascent: One player runs ship winch; other rides Bell, defending loot from sharks
  - Return: Sail back to port while a storm chases you
- Two-Layer Physics System
  - Surface (Newtonian): Ship is a rigid body driven by OceanSolver (Gerstner waves); players walk on deck using base-velocity impart to avoid sliding
  - Abyss (6DOF): Below sea level, gravity off; players swim with 6 degrees of freedom; lighting shifts to thalassophobia mode (darkness, flashlight/bioluminescence)

### Technical Architecture (C++)
- FOceanSolver (struct)
  - CPU wave height sampling using summed Gerstner waves; ensures server-side physics matches client visuals
- AMasterShip (Pawn + Interface)
  - Components: Hull mesh, 4 pontoons, spring arm, follow camera
  - Input: Enhanced Input (Throttle, Rudder)
  - Physics: 4-point buoyancy in Tick
  - Interface: Implements `IInteractableInterface` for possession/interaction
- AMarineCharacter (Character)
  - Movement: Tuned for moving floors (ImpartVelocity true)
  - Input: WASD, Mouse, E (Interact)
  - Interaction: Raycasts; executes interface on hit actor
- IInteractableInterface (Interface)
  - `Interact(APawn* Instigator)` standardizes interactions across Ships, Winches, Cannons, Loot
- Safe Winch Architecture
  - Problem: Long physics constraints (500m+) become unstable
  - Solution:
    - Visual: UCableComponent connects Ship and Bell (no collision)
    - Logic: Bell is kinematic; moves via Timeline/Interp based on Winch state
    - Physics: Enabled only for short distances between Bell and loot

### Visual Direction (UE 5.7)
- Lighting Strategy
  - Surface: Lumen GI + Volumetric Clouds
  - Deep: MegaLights (5.7) for 1000+ shadow-casting lights; glowing fish and flares cast dynamic shadows
  - Absorption: Material parameter collection fades colors by depth (red disappears at ~10m, blue at ~100m)
- Biomes
  - Shallows (0–50m): Bright, safe (Nanite coral, white sand)
  - Kelp Forest (50–150m): Murky, low visibility (tall Nanite kelp, current motion)
  - Midnight Zone (150m+): Pitch black (basalt rocks, “snow” particles, bioluminescent flora)

### Development Roadmap (Solo Dev)
- Phase 1: Foundation (Current)
  - [x] OceanSolver + MasterShip buoyancy
  - [x] Enhanced Input for Ship (WASD)
  - [x] MarineCharacter setup
  - [x] Interaction interface for walking/sailing swap
- Phase 2: Mechanic (Next)
  - [ ] Winch: Visual cable + Bell actor
  - [ ] Loot: `BP_Artifact` with `UPhysicsHandle`
  - [ ] Inventory: Simple slots for small items
- Phase 3: World
  - [ ] Landmass: Sculpt Spawn Island (Safe Zone)
  - [ ] PCG: Scatter graph to populate ocean floor (rocks/coral)
  - [ ] Atmosphere: Volumetric fog + post-process depth fading
- Phase 4: Threat
  - [ ] AI: Boids (fish) + hostile shark (NavMesh invoker)
  - [ ] Damage: Ship hull health + repair hammer
