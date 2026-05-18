extends Node2D

const CommanderScene := preload("res://scenes/commander.tscn")

@onready var world_root: Node2D = $WorldRoot
@onready var camera: Camera2D = $Camera2D

var _my_id: String = ""
var _commanders: Dictionary = {}
var _world_size := Vector2(10000, 10000)

func _ready() -> void:
	WSClient.welcome.connect(_on_welcome)
	WSClient.snapshot.connect(_on_snapshot)

func _on_welcome(data: Dictionary) -> void:
	_my_id = str(data.get("player_id", ""))
	var spawn: Array = data.get("spawn", [0, 0])
	camera.position = Vector2(spawn[0], spawn[1])
	var world: Array = data.get("world", [10000, 10000])
	_world_size = Vector2(world[0], world[1])
	print("[Game] welcome id=", _my_id, " spawn=", camera.position)

func _on_snapshot(data: Dictionary) -> void:
	var players: Array = data.get("players", [])
	var seen := {}
	for raw in players:
		var p: Dictionary = raw
		var id := str(p.get("id", ""))
		if id.is_empty():
			continue
		seen[id] = true
		var c: Node2D = _commanders.get(id)
		if c == null:
			c = CommanderScene.instantiate()
			world_root.add_child(c)
			_commanders[id] = c
		var primary: Array = p.get("primary", [255, 255, 255])
		var outline: Array = p.get("outline", [128, 128, 128])
		c.set_colors(
			Color8(primary[0], primary[1], primary[2]),
			Color8(outline[0], outline[1], outline[2])
		)
		var pos: Array = p.get("pos", [0, 0])
		c.set_target_pos(Vector2(pos[0], pos[1]))
	for id in _commanders.keys():
		if not seen.has(id):
			_commanders[id].queue_free()
			_commanders.erase(id)

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_RIGHT:
			var world_pos := get_global_mouse_position()
			WSClient.send({"type": "move", "target": [world_pos.x, world_pos.y]})
	elif event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_R:
			WSClient.send({"type": "respawn"})
