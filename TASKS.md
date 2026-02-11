# Project Abyss - Development Tasks

This file tracks development tasks and issues for the Project Abyss team.

## Task Status Legend
- 🔴 Not Started
- 🟡 In Progress  
- 🟢 Complete
- ⏸️ Blocked

---

## Independent Tasks (Non-Blocking)

These tasks can be worked on independently without dependencies on other features or systems.

### Task #001: Add Git Commit Message Template
**Status:** 🟡 Not Started  
**Assigned to:** @JMiloMartini  
**Priority:** Low  
**Type:** Developer Tooling  

**Description:**  
Create a `.gitmessage` template file to standardize commit messages across the team. This will help maintain consistent commit history and make it easier to track changes.

**Requirements:**
- Create a `.gitmessage` file in the repository root
- Include template sections for:
  - Type (feat/fix/docs/style/refactor/test/chore)
  - Scope (e.g., ship, character, ocean, interaction)
  - Short description (imperative mood)
  - Detailed description (optional)
  - References to issues/tasks (optional)
- Add instructions to README.md on how to configure Git to use the template
- Template should follow conventional commits format

**Benefits:**
- Improves commit message quality
- Makes git log more readable
- Helps with automated changelog generation
- Sets best practices for the team

**Acceptance Criteria:**
- [ ] `.gitmessage` file exists in repository root
- [ ] Template follows conventional commits format
- [ ] README.md includes setup instructions
- [ ] Template is easy to use and understand

**Estimated Effort:** 1-2 hours  
**Dependencies:** None

---

## Core Feature Tasks

### Task #002: Exit Ship Interaction
**Status:** 🟢 Complete  
**Assigned to:** Core Team  
**Priority:** High  
**Type:** Feature  

**Description:**  
Implement ability to exit the ship and return control to the marine character.

**Requirements:**
- Add keybinding to exit ship (E key when possessed)
- Cache reference to character pawn during possession
- Restore character control and camera on exit
- Restore character input mapping context

**Status:** Completed in recent commits

---

## Future Tasks

### Task #003: Winch System Implementation
**Status:** 🔴 Not Started  
**Priority:** High  
**Type:** Feature  
**Dependencies:** Exit Ship Interaction (Task #002)

**Description:**  
Implement the diving bell winch system for artifact retrieval.

### Task #004: Interaction Prompts UI
**Status:** 🔴 Not Started  
**Priority:** Medium  
**Type:** UX Polish  

**Description:**  
Add visual prompts when player can interact with objects (e.g., "Press E to Take Wheel").

### Task #005: Loot System
**Status:** 🔴 Not Started  
**Priority:** High  
**Type:** Feature  

**Description:**  
Implement artifact/loot pickup system with physics handling.

---

## Completed Tasks

- ✅ OceanSolver + MasterShip buoyancy
- ✅ Enhanced Input for Ship (WASD)  
- ✅ MarineCharacter setup
- ✅ Interaction interface implementation
- ✅ Ship possession/control handshake
- ✅ Exit ship functionality

---

## Notes

- Tasks marked as "Independent" can be worked on in parallel with other development
- High priority tasks should be addressed before medium/low priority
- Update task status regularly to reflect current progress
- Add new tasks to appropriate sections as they are identified
