# TODO

<!-- dear AGENT:
Do these in written order. Make commit before moving on to next task. Iterate until there are no tasks left. Check them off as you go. If anything is unclear, add questions for USER to answer to the end of the file and move on.
-->

- [x] Make aberrations spawn in the gameplay loop, only spawning them inside the view frustum, reducing the mouse sensitivity to a low value while it is in its spawn animation. Spawn one every 5-10 seconds and only allow 5 to be existing at a time. Do not spawn any at game start.
- [x] Make color overlay transition only start to animate T-5 seconds from scene switch, also instead of a solid color overlay, add a new shader parameter in palette.rs to darken input color before quantization
- [x] Add svg rendering crate `Weasy666/bevy_svg`. Add "assets/vector_sprites/creeper_A.svg" as a banner in the menu screen
- [x] Pressing escape in-game should bring up a pause menu with various game stats and a button to Continue, and a button to Exit to Menu.

- [x] Use the github mcp or the github cli (`gh`) to check the status of the CI pipelines, fix any issues
- [ ] Add Puppeteer plugin that can automate basic gameplay, and take screenshots, to test gameplay functionality. Make a test and run it.

## Questions
<!-- questions for USER go here: -->