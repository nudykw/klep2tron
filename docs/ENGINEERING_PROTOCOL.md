# 🛠 Engineering Design Protocol (High-Performance Approach)

> [!NOTE]
> This document is part of the two-stage design system in `klep2tron`.
> 1. **[Product Protocol](PRODUCT_PROTOCOL.md)** — answers "What are we building and why?".
> 2. **Engineering Protocol** (this file) — answers "How to implement this efficiently?".

This document describes the quality standard for implementing complex and interactive systems in the `klep2tron` project. We use this approach to keep the interface responsive (60+ FPS) and the code extensible.

---

## 1. Golden Rule: Architecture Before Code

Any task more complex than "fixing a typo" must go through an architecture design phase.

### How to initiate the approach:
1. Copy the **[Engineering Template](templates/ENGINEERING_TEMPLATE.md)** to a plan file (e.g., `plans/Feature_Architecture.md`).
2. Fill in the key technical points.
3. Send me the link and say: 
> *"Let's apply the Engineering Protocol: supplement this plan with architecture and diagrams."*

---

## 2. Design Principles

### A. Hybrid Path (Fast Path vs. Full Path)
For any heavy operation (mesh slicing, pathfinding, physics simulation), we divide the logic into two streams:

1.  **Fast Path (Preview):** 
    *   Runs every frame during interaction (e.g., drag-and-drop).
    *   **Goal:** Instant visual feedback (<16ms).
    *   **Method:** Simplified geometry, lines only (Gizmos), no creation of new entities or meshes.
2.  **Full Path (Action):**
    *   Runs once upon confirmation (Confirm).
    *   **Goal:** High precision and creation of persistent data.
    *   **Method:** Mesh generation, writing to files, creating ECS entities.

### B. Visual State Indication
*   **Orange (Preview):** "What happens if I do this" state. Not saved.
*   **Red/White (Final):** Committed state. Written to components.

---

## 3. Mandatory Planning Artifacts

Before writing code, I (the AI) must provide:

1.  **Mermaid Diagram (Architecture):** A scheme of how data flows between Bevy systems (`Update` vs `PostUpdate`).
2.  **State Machine:** If an action has confirmation/cancellation, the transition logic must be described.
3.  **Performance Targets:** Estimation of how long the operation should take on the fast and slow paths.

---

## 4. User Checklist (How to review the plan)

When I send the `implementation_plan.md`, check it against these points:
- [ ] Are the interactive part and heavy calculations separated?
- [ ] Is there a visualization (diagram)?
- [ ] Is it clear how the user can cancel the action before confirmation?
- [ ] Are "degenerate" cases handled (empty meshes, zero values, etc.)?

---

## 5. Reference Implementation Example
The model for this approach is commit `a4b7509c94f5da40281af65902c42fae4ee77af8` (ActorEditor slicing optimization). If I propose a solution below this standard, demand a rework according to the "Engineering Protocol".
