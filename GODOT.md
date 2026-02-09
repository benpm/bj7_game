# Godot Quick Reference

## File Types

- `.godot/` - Editor-generated metadata/cache directory. Gitignore it.
- `project.godot` - Project root config file. Defines project settings, autoloads, input maps. Its presence marks a directory as a Godot project.
- `.tscn` - Text Scene. Human-readable scene tree (nodes, properties, connections). Version-control friendly.
- `.scn` - Binary Scene. Same as `.tscn` but binary-encoded. Smaller, not diffable.
- `.tres` - Text Resource. Human-readable resource (materials, themes, custom resources).
- `.res` - Binary Resource. Same as `.tres` but binary-encoded.
- `.gd` - GDScript source file.
- `.gdshader` / `.gdshaderinc` - Godot shader / shader include files.
- `.import` - Auto-generated import metadata for assets. Gitignore these or let Godot regenerate them.
- `.uid_cache` / `.godot_script_cache` - Editor caches in `.godot/`. Always gitignored.
- `.pck` - Packed export file containing all project resources.

## GDScript Essentials

### Script Structure
```gdscript
class_name MyClass          # Optional global class name
extends Node2D              # Parent class (required for attached scripts)

# Signals
signal health_changed(new_value: int)

# Exports (editable in Inspector)
@export var speed: float = 200.0
@export var max_health: int = 100
@export_range(0, 100) var volume: int = 50

# Onready (resolved when node enters tree, after _ready of children)
@onready var sprite: Sprite2D = $Sprite2D
@onready var label := $UI/Label as Label

# Regular variables
var health: int = 100
var _private_var := 0       # Convention: underscore prefix for private
```

### Lifecycle Callbacks (call order)
```
_init()          # Constructor. Called on object creation.
_enter_tree()    # Node added to scene tree (before children).
_ready()         # Node and ALL children are in the tree. Runs once.
_process(delta)  # Called every frame. delta = seconds since last frame.
_physics_process(delta)  # Called every physics tick (default 60/s). Use for movement/physics.
_input(event)    # Receives all InputEvents.
_unhandled_input(event)  # InputEvents not consumed by _input or GUI.
_exit_tree()     # Node about to leave the scene tree.
```

### Types & Annotations
```gdscript
var x: int = 5
var name: String = "hello"
var pos: Vector2 = Vector2(10, 20)
var items: Array[String] = ["a", "b"]
var dict: Dictionary = {"key": "value"}
var node: Node = null

# Type inference
var y := 5              # Inferred as int
var v := Vector2.ZERO   # Inferred as Vector2

# Constants
const MAX_SPEED := 400.0
enum State { IDLE, RUNNING, JUMPING }

# Functions
func move(direction: Vector2, delta: float) -> void:
    position += direction * speed * delta

func get_info() -> String:
    return "Player"
```

### Signals
```gdscript
# Declare
signal died
signal score_changed(new_score: int)

# Emit
died.emit()
score_changed.emit(42)

# Connect (in code)
button.pressed.connect(_on_button_pressed)
enemy.died.connect(_on_enemy_died)

# Disconnect
button.pressed.disconnect(_on_button_pressed)
```

### Node Access
```gdscript
$ChildNode                  # Get direct child by name
$Path/To/DeepChild          # Get nested child
get_node("ChildNode")       # Equivalent to $ChildNode
get_parent()                # Parent node
get_tree()                  # SceneTree
get_tree().root             # Root viewport
owner                       # Scene root this node belongs to
```

### Common Patterns
```gdscript
# Load and instantiate a scene
var scene := preload("res://scenes/bullet.tscn")
var bullet := scene.instantiate()
add_child(bullet)

# Change scene
get_tree().change_scene_to_file("res://scenes/game_over.tscn")

# Groups
add_to_group("enemies")
get_tree().get_nodes_in_group("enemies")
get_tree().call_group("enemies", "take_damage", 10)

# Timers
await get_tree().create_timer(2.0).timeout

# Tween
var tween := create_tween()
tween.tween_property(self, "modulate:a", 0.0, 1.0)
```

### Static Typing Benefits
- Catches type errors at write-time in the editor.
- Enables better autocompletion.
- GDScript runs ~10-20% faster with typed code (avoids dynamic dispatch).
- Use `-> void` on functions that return nothing.

## Debugging

### Print Functions
```gdscript
print("basic output")                        # Prints to Output panel and stdout
print_rich("[color=red]error[/color]")        # BBCode-colored output
prints("spaced", "values")                   # Space-separated
printt("tab", "separated")                   # Tab-separated
printerr("goes to stderr")                   # Prints to stderr
print_debug("with stack info")               # Prints with script/line info

push_error("serious problem")                # Error in Output + Debugger (red)
push_warning("potential issue")              # Warning in Output (yellow)
```

### Assertions
```gdscript
assert(health > 0, "Health must be positive")  # Debug-only. Removed in release builds.
```

### Breakpoints
- Click the left gutter (line numbers) in the script editor to toggle breakpoints (red dot).
- Or use the `breakpoint` keyword in code (persists in version control).
- Breakpoints do NOT work inside threads.

### Debugger Panel (bottom of editor)
- **Stack Variables**: Inspect locals/members/globals when paused at breakpoint.
- **Step Into** (F11): Enter function calls line by line.
- **Step Over** (F10): Execute next line, skip into function internals.
- **Continue** (F12): Resume until next breakpoint.
- **Break**: Pause execution immediately.

### Visual Debug Flags (CLI or Editor)
- `--debug-collisions` - Render collision shapes.
- `--debug-navigation` - Render navigation polygons.
- `--debug-paths` - Render path lines.
- `--debug-avoidance` - Render avoidance debug visuals.
- `--debug-canvas-item-redraw` - Highlight canvas items on redraw.

### Profiler
- Open from **Debugger > Profiler** tab while game runs.
- Shows per-frame time breakdown by function.
- **Monitor** tab tracks FPS, memory, draw calls, physics objects, etc.

### Remote Scene Inspector
- While the game runs, the **Remote** tab in the Scene dock shows the live scene tree.
- Select nodes to inspect/edit their properties in real time.

## Godot CLI

### Running
```sh
godot                               # Run project in current directory
godot --path /my/project             # Run project at specified path
godot -e                             # Open editor
godot -e --path /my/project          # Open editor for specific project
godot --scene res://scenes/test.tscn # Run a specific scene
godot -p                             # Open Project Manager
```

### Debugging
```sh
godot -d                             # Run with local stdout debugger
godot --debug-collisions             # Visualize collision shapes
godot --debug-navigation             # Visualize navigation meshes
godot --remote-debug tcp://127.0.0.1:6007  # Remote debugger
godot -v                             # Verbose output
godot -q                             # Quiet mode (errors only)
godot --print-fps                    # Print FPS to stdout
godot --max-fps 30                   # Cap framerate
godot --fixed-fps 60                 # Force fixed FPS
godot --time-scale 2.0               # Speed up game time
godot --log-file debug.log           # Write log to file
```

### Exporting
```sh
godot --headless --export-release "Windows Desktop" ./build/game.exe
godot --headless --export-debug "Windows Desktop" ./build/game_debug.exe
godot --headless --export-pack "Windows Desktop" ./build/game.pck
```

### Scripting / Tools
```sh
godot -s my_script.gd                # Run a standalone script
godot -s my_script.gd --check-only   # Parse script for errors only
godot --headless --import             # Import resources then quit
godot --headless --quit               # Start and immediately quit (useful for CI import)
godot --doctool ./docs                # Dump API reference to XML
godot --convert-3to4                  # Convert Godot 3.x project to 4.x
godot --build-solutions               # Build C# solutions
```

### Display
```sh
godot -f                             # Fullscreen
godot -m                             # Maximized
godot -w                             # Windowed
godot --resolution 1920x1080         # Set resolution
godot --headless                     # No window, no audio (CI/servers)
godot --rendering-method gl_compatibility  # Force compatibility renderer
```
