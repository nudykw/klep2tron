# 📋 Product Design Protocol (Feature & UX Logic)

> [!NOTE]
> This document is part of the two-stage design system in `klep2tron`.
> 1. **Product Protocol** (this file) — answers "What are we building and why?".
> 2. **[Engineering Protocol](ENGINEERING_PROTOCOL.md)** — answers "How to implement this efficiently?".

This document describes the standard for defining functional requirements and user interaction logic in the `klep2tron` project.

---

## 1. Goal: Defining "What" and "Why"

Before thinking about optimization and code, we must clearly understand what user problem we are solving and what the final result will look like.

### How to initiate the approach:
1. Copy the **[Product Template](templates/PRODUCT_TEMPLATE.md)** to a new file (e.g., `docs/design/new_feature.md`).
2. Fill in the fields in square brackets.
3. Send me the link to the file and say: 
> *"Let's apply the Product Protocol for this description."*

---

## 2. Product Description Principles

### A. Description through User Stories
Instead of "add a button", we write:
*   **Who:** (e.g., Editor User).
*   **Action:** (wants to select triangles with a lasso).
*   **Result:** (to quickly reassign them to the head without moving sliders).

### B. Defining Constraints
*   What the system **SHOULD NOT** do? (e.g., "not pierce the model through when selecting").
*   What are the dependencies? (e.g., "works only if the slicer is locked").

### C. Transition Logic (States)
We describe how the program's behavior changes:
*   What happens when switching modes (Auto -> Manual)?
*   Is confirmation needed (Confirm/Cancel)?
*   What data is "frozen" and what is reset?

---

## 3. Mandatory Artifacts

As a result of product planning, I (the AI) must provide:
1.  **Acceptance Criteria:** A list of items by which you will verify that the task is completed.
2.  **UI/UX Draft:** Description of button placement, tooltips, and visual effects (Wireframe or text description).
3.  **Task Breakdown:** A list of small sub-tasks (e.g., 14.1, 14.2...) that can be performed independently.

---

## 4. User Checklist

Verify the product description against these points:
- [ ] Is the behavior in non-standard situations (miss-click, cancellation, error) described?
- [ ] Do all new buttons and controls have a logical place in the UI?
- [ ] Is it clear what data will be saved to the project file?
- [ ] Are there no logical contradictions with existing features?

---

## 5. Reference Implementation Example
The model for this approach is commit `741adc1674759ed390c665bb1d611139dac06086` (Manual Mesh Slicing Roadmap).
