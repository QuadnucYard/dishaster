extends Node2D

func _enter_tree() -> void:
	# Apply random modulate with fine colors: random hue, high saturation and lightness for vibrant appearance
	self.modulate = Color.from_ok_hsl(randf(), randf_range(0.7, 1.0), randf_range(0.6, 0.9))
