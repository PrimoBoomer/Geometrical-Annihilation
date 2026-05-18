extends Node

signal welcome(data)
signal snapshot(data)
signal disconnected

const DEFAULT_URL := "ws://127.0.0.1:8080/"

var _peer := WebSocketPeer.new()
var _open := false

func _ready() -> void:
	var url := _resolve_url()
	print("[WSClient] connecting to ", url)
	var err := _peer.connect_to_url(url)
	if err != OK:
		push_error("[WSClient] connect_to_url failed: %s" % err)

func _resolve_url() -> String:
	if OS.has_feature("web"):
		var search: String = JavaScriptBridge.eval("window.location.search", true)
		if typeof(search) == TYPE_STRING and search.length() > 1:
			var params := search.substr(1).split("&", false)
			for kv in params:
				var parts := kv.split("=", true, 1)
				if parts.size() == 2 and parts[0] == "ws":
					return parts[1].uri_decode()
	return DEFAULT_URL

func _process(_delta: float) -> void:
	_peer.poll()
	var state := _peer.get_ready_state()
	if state == WebSocketPeer.STATE_OPEN:
		if not _open:
			_open = true
			print("[WSClient] connected")
			send({"type": "hello"})
		while _peer.get_available_packet_count() > 0:
			var pkt := _peer.get_packet()
			var txt := pkt.get_string_from_utf8()
			var data: Variant = JSON.parse_string(txt)
			if typeof(data) != TYPE_DICTIONARY:
				continue
			match data.get("type", ""):
				"welcome":
					welcome.emit(data)
				"snapshot":
					snapshot.emit(data)
	elif state == WebSocketPeer.STATE_CLOSED:
		if _open:
			_open = false
			print("[WSClient] disconnected")
			disconnected.emit()

func send(msg: Dictionary) -> void:
	if _peer.get_ready_state() != WebSocketPeer.STATE_OPEN:
		return
	_peer.send_text(JSON.stringify(msg))
