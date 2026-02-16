# TODO

> Do these in written order. Pull, commit, and push before moving on to next task. Iterate until there are no tasks left. Check them off as you go, moving them to [done](#done). If anything is unclear, add questions for USER to answer in [Questions](#questions)

## Ready

> These are ready for work, probably.

- [ ] Use `FeatherCursor.png` as the mouse cursor image when in dispel mode.
- [ ] In the main menu, show the `splash.png` texture, tiled, in the background. 


## WIP

> These are WIP and should not be worked on until moved to [done](#done)



## Questions
<!-- questions for USER go here: -->


## Done

- [x] Fix camera remaining locked after dialog completes
- [x] Indent response buttons, NPC portrait next to text box, animated button height grow-in
- [x] Configure and test the CI pipeline for web deployment. It should run on every commit.
- [x] NPC billboard sprites from .ron config, aberration size property configurable in types.ron
- [x] Recursive dialog tree from `assets/defs/dialog.ron` with Player response buttons, cursor management, hover effects
- [x] Set the bevy `UiScale` to 2
- [x] Layered sprite sheet aberrations defined by .ron files (BaseAnimAberrationSheet + GreenFace overlay)
- [x] RPG-style dialog system with bevy_text_animation, embedded dialog_tree.ron, NPC interaction with E key
- [x] Use 9-slice textbox texture for buttons and pause menu modal
- [x] Add a point light source to the player
- [x] Separate out the transition logic from environment.rs into a new struct, allowing scene transitions to be triggered independently
- [x] Use font assets/fonts/sd-auto-pilot.ttf for the font for all UI
- [x] Update palette.rs to work on the entire game's full color output, including UI
- [x] Make aberrations spawn in the gameplay loop, only spawning them inside the view frustum, reducing the mouse sensitivity to a low value while it is in its spawn animation. Spawn one every 5-10 seconds and only allow 5 to be existing at a time. Do not spawn any at game start.
- [x] Make color overlay transition only start to animate T-5 seconds from scene switch, also instead of a solid color overlay, add a new shader parameter in palette.rs to darken input color before quantization
- [x] Add svg rendering crate `Weasy666/bevy_svg`. Add "assets/vector_sprites/creeper_A.svg" as a banner in the menu screen
- [x] Pressing escape in-game should bring up a pause menu with various game stats and a button to Continue, and a button to Exit to Menu.
- [x] Use the github mcp or the github cli (`gh`) to check the status of the CI pipelines, fix any issues
- [x] In order to dispel the aberrations, the player should be able to left click, which will enable dispel mode. In dispel mode, the camera will lock in place, and the mouse cursor should be shown. Then, if the player left clicks and holds, a segmented line should be drawn, with each segment being an additional time step that the mouse button is held. If the mouse cursor gets near to the first click point, and the loop is closed, aberrations with their screen-space centroid inside the polygon are dispelled.