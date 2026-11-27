# Ballpark, interface and art direction

Build an Ubuntu-native 3D ballpark where a manager can read both sides’ game state and see baseball happen. Preserve the actual printed faces and common back. Let the design rounds determine camera, framing, lighting, scale and visual style. Optimize the view around the next meaningful decision. At a Glove decision, show which threats can be stopped and the projected consequences. At a buy, keep budget, candidate, cost, immediate refill and demotion legible together.

Distinguish threatened hits from resolved action. A batter can wait at home while the opponent responds; after resolution the ball can fly and runners advance in order. Support quick completion of motion and a stable final state. Never hide a legal decision under an animation. A projection must account for board position without revealing private cards. White, blue and red runners need distinguishable silhouettes or speed marks at playing scale. Runner speed and Natural/Robot/Cyborg are separate dimensions.

For each new 2D graphic, present at least three competent, materially different alternatives made in VectorCraft. For each layout or composite UI, make at least three alternatives using both VectorCraft and FigCraft. Compare them in the same real decision at intended size. Show the owner the alternatives and obtain selection before application implementation. Effects belong in EffectCraft when motion is being authored. Preserve editable source and inspect the selected result in the native scene.

For new 3D pieces agree on the piece’s role and scale, then create consistent front, back, left and right turnaround portraits when modeling requires them. Maintain the same proportions, orientation convention, materials and base across views. Choose Blender modeling or Tripo with the owner for that asset. Tripo submission, external uploads and spend are separate from local design. Inspect the resulting piece from the game camera, alongside the other speeds and the cards, and let the owner judge the combined presentation.

## Selected direction for Glove and the Buy Round

The owner selected the [combined manager study](../../art/manager-review/COMBINED.md) on 2026-11-27. Use a high field camera for live decisions, warm scorecard panels for comparing consequences, and player art in six stable market slots. A low camera can follow a resolved hit before the next decision. The [review handoff](../../art/manager-review/HANDOFF.md) supplies the two positions, the editable study restoration steps and the shared-code trace. Native play will determine whether speed, revenue and pinch-hit eligibility also need to appear directly on each market offer.

## Authoring tools

Start a trusted Codex session or Claude Code from the project root. The project configuration registers VectorCraft, FigCraft and EffectCraft by game name, reads the checkout and writes editable art under art/. Blender uses the shared local Blender MCP bridge (mcp-for-blender 2.3.1, uvx, Python 3.11). Enable the companion in the intended Blender session before scene work. Reopen the project client to load newly added MCP definitions. Keep private reference imports and exports under owner custody.
