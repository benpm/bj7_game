# TODO

> Do these in written order. Make commit before moving on to next task. Iterate until there are no tasks left. Check them off as you go, moving them to [done](#done). If anything is unclear, add questions for USER to answer in [Questions](#questions)

## Ready

> These are ready for work, probably.





## WIP

> These are WIP and should not be worked on until moved to [done](#done)

- [ ] Add Puppeteer plugin that can automate basic gameplay, and take screenshots, to test gameplay functionality. Make a test and run it.
  - **Prerequisites**: `cargo install --locked trunk` (trunk not currently installed; wasm target + Node.js v25 + npm 11 already present)
  - **Files to create**: `tests/puppeteer/package.json` (puppeteer dep), `tests/puppeteer/gameplay.test.mjs` (test script)
  - **Gitignore**: Add `tests/puppeteer/node_modules/` and `tests/puppeteer/screenshots/`
  - **Test flow**: `trunk build` → serve `dist/` via Node http → headless Chrome with WebGL → navigate menu → click Play (canvas center ~640,360) → take screenshots at each stage (menu, gameplay, pause) → verify non-trivial file sizes (>10KB)
  - **Puppeteer config**: `headless: 'shell'`, `args: ['--enable-webgl', '--no-sandbox', '--use-gl=angle']`, viewport 1280x720
  - **Run**: `cd tests/puppeteer && npm install && npm test`

## Questions
<!-- questions for USER go here: -->


## Done

- [x] Separate out the transition logic from environment.rs into a new struct, allowing scene transitions to be triggered independently
- [x] Use font assets/fonts/sd-auto-pilot.ttf for the font for all UI
- [x] Update palette.rs to work on the entire game's full color output, including UI
- [x] Make aberrations spawn in the gameplay loop, only spawning them inside the view frustum, reducing the mouse sensitivity to a low value while it is in its spawn animation. Spawn one every 5-10 seconds and only allow 5 to be existing at a time. Do not spawn any at game start.
- [x] Make color overlay transition only start to animate T-5 seconds from scene switch, also instead of a solid color overlay, add a new shader parameter in palette.rs to darken input color before quantization
- [x] Add svg rendering crate `Weasy666/bevy_svg`. Add "assets/vector_sprites/creeper_A.svg" as a banner in the menu screen
- [x] Pressing escape in-game should bring up a pause menu with various game stats and a button to Continue, and a button to Exit to Menu.
- [x] Use the github mcp or the github cli (`gh`) to check the status of the CI pipelines, fix any issues
- [x] In order to dispel the aberrations, the player should be able to left click, which will enable dispel mode. In dispel mode, the camera will lock in place, and the mouse cursor should be shown. Then, if the player left clicks and holds, a segmented line should be drawn, with each segment being an additional time step that the mouse button is held. If the mouse cursor gets near to the first click point, and the loop is closed, aberrations with their screen-space centroid inside the polygon are dispelled.