extends Node2D

const RADIUS := 40.0
const LERP_SPEED := 8.0

var _primary: Color = Color.WHITE
var _outline: Color = Color.GRAY
var _target_pos: Vector2 = Vector2.ZERO
var _has_pos := false

func set_colors(primary: Color, outline: Color) -> void:
	_primary = primary
	_outline = outline
	queue_redraw()

func set_target_pos(p: Vector2) -> void:
	if not _has_pos:
		position = p
		_has_pos = true
	_target_pos = p

func _process(delta: float) -> void:
	if _has_pos:
		position = position.lerp(_target_pos, clampf(LERP_SPEED * delta, 0.0, 1.0))

func _draw() -> void:
	var pts := PackedVector2Array()
	for i in range(3):
		var angle: float = -PI * 0.5 + i * TAU / 3.0
		pts.append(Vector2(cos(angle), sin(angle)) * RADIUS)
	pts.append(pts[0])
	draw_polyline(pts, _primary, 4.0, true)
	var inner := PackedVector2Array()
	for p in pts:
		inner.append(p * 0.7)
	draw_polyline(inner, _outline, 2.0, true)
