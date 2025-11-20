@tool
extends EditorScript

## Generate dish prefabs from texture atlas
## Run this script from Editor -> Run Script in Godot

const TEXTURES_DIR = "res://assets/prefabs/dishes/_textures"
const PREFABS_DIR = "res://assets/prefabs/dishes"
const DISH_TEXTURES_DIR = "res://assets/sprites/dishes"
const ATLAS_PNG_PATH = "res://assets/sprites/dishes_atlas.png"
const TILE_SIZE = 64
const ATLAS_COLS = 8

func _run():
	print("=== Dish Prefab Generator ===\n")

	# Step 1: Collect images and create atlas
	var images = collect_dish_images()
	if images.is_empty():
		print("No dish images found!")
		return

	print("Found %d dish images" % images.size())

	# Step 2: Create atlas
	var atlas_map = create_atlas(images)
	if atlas_map.is_empty():
		return

	# Step 3: Generate or update prefabs
	print("\nGenerating/updating prefabs...")
	var created = 0
	var updated = 0
	var skipped = 0

	for dish_id in atlas_map:
		var pos = atlas_map[dish_id]
		var result = generate_or_update_prefab(dish_id, pos.x, pos.y)
		match result:
			"created":
				created += 1
			"updated":
				updated += 1
			"skipped":
				skipped += 1

	print("\nResults: Created %d, Updated %d, Skipped %d" % [created, updated, skipped])
	print("\n=== Done! ===")
	print("Atlas PNG: %s" % ATLAS_PNG_PATH)
	print("Dish Textures: %s" % DISH_TEXTURES_DIR)
	print("Prefabs: %s" % PREFABS_DIR)


func collect_dish_images() -> Array:
	"""Collect all PNG dish images from _textures folder"""
	var images = []
	var dir = DirAccess.open(TEXTURES_DIR)

	if dir == null:
		print("Error: Cannot open directory %s" % TEXTURES_DIR)
		return images

	dir.list_dir_begin()
	var file_name = dir.get_next()

	while file_name != "":
		if not dir.current_is_dir() and file_name.ends_with(".png"):
			var dish_id = file_name.get_basename()
			images.append(dish_id)
		file_name = dir.get_next()

	dir.list_dir_end()
	images.sort()
	return images


func create_atlas(images: Array) -> Dictionary:
	"""Create texture atlas from individual dish images"""
	var atlas_map = {}
	var rows = ceili(float(images.size()) / ATLAS_COLS)
	var atlas_width = ATLAS_COLS * TILE_SIZE
	var atlas_height = rows * TILE_SIZE

	print("\nCreating atlas: %dx%d (%dx%d tiles)" % [atlas_width, atlas_height, ATLAS_COLS, rows])

	# Create blank atlas image
	var atlas_image = Image.create(atlas_width, atlas_height, false, Image.FORMAT_RGBA8)
	atlas_image.fill(Color(0, 0, 0, 0))

	# Place each dish image
	for idx in range(images.size()):
		var dish_id = images[idx]
		var row = floori(float(idx) / ATLAS_COLS)
		var col = idx % ATLAS_COLS
		var pos_x = col * TILE_SIZE
		var pos_y = row * TILE_SIZE

		# Load source image
		var source_path = "%s/%s.png" % [TEXTURES_DIR, dish_id]
		var source_image = Image.load_from_file(source_path)

		if source_image == null:
			print("Warning: Failed to load %s" % source_path)
			continue

		# Check size
		if source_image.get_width() != TILE_SIZE or source_image.get_height() != TILE_SIZE:
			print("Warning: %s is %dx%d, expected %dx%d - resizing" % [
				dish_id, source_image.get_width(), source_image.get_height(), TILE_SIZE, TILE_SIZE
			])
			source_image.resize(TILE_SIZE, TILE_SIZE, Image.INTERPOLATE_LANCZOS)

		# Blit to atlas
		atlas_image.blit_rect(source_image, Rect2i(0, 0, TILE_SIZE, TILE_SIZE), Vector2i(pos_x, pos_y))
		atlas_map[dish_id] = Vector2i(col, row)
		print("  %s: atlas position (%d, %d)" % [dish_id, col, row])

	# Save atlas PNG
	var err = atlas_image.save_png(ATLAS_PNG_PATH)
	if err != OK:
		print("Error: Failed to save atlas PNG: %s" % err)
		return {}

	print("\nAtlas PNG saved: %s" % ATLAS_PNG_PATH)

	# Save individual dish texture resources
	print("\nSaving individual dish textures...")
	for dish_id in atlas_map:
		var pos = atlas_map[dish_id]
		var x = pos.x * TILE_SIZE
		var y = pos.y * TILE_SIZE

		var dish_atlas_texture = AtlasTexture.new()
		dish_atlas_texture.atlas = load(ATLAS_PNG_PATH)
		dish_atlas_texture.region = Rect2(x, y, TILE_SIZE, TILE_SIZE)

		var dish_texture_path = "%s/%s.tres" % [DISH_TEXTURES_DIR, dish_id]
		err = ResourceSaver.save(dish_atlas_texture, dish_texture_path)
		if err != OK:
			print("  Error saving %s: %s" % [dish_texture_path, err])
		else:
			print("  Saved %s.tres" % dish_id)

	print("\nAll individual textures saved!")
	return atlas_map


func generate_or_update_prefab(dish_id: String, col: int, row: int) -> String:
	"""Generate or update a dish prefab scene"""
	var prefab_path = "%s/%s.tscn" % [PREFABS_DIR, dish_id]
	var x = col * TILE_SIZE
	var y = row * TILE_SIZE

	# Check if prefab exists
	if ResourceLoader.exists(prefab_path):
		return update_existing_prefab(prefab_path, dish_id, x, y)
	else:
		return create_new_prefab(prefab_path, dish_id, x, y)


func create_new_prefab(prefab_path: String, dish_id: String, x: int, y: int) -> String:
	"""Create a new prefab scene"""
	# Create scene tree
	var area = Area2D.new()
	area.name = dish_id.capitalize().replace(" ", "")

	# Load the individual texture resource
	var dish_texture_path = "%s/%s.tres" % [DISH_TEXTURES_DIR, dish_id]
	var dish_texture = load(dish_texture_path)

	if dish_texture == null:
		print("  Error: Failed to load texture resource %s" % dish_texture_path)
		return "error"

	# Create sprite
	var sprite = Sprite2D.new()
	sprite.name = "Sprite2D"
	sprite.scale = Vector2(0.5, 0.5)
	sprite.texture = dish_texture
	area.add_child(sprite, true)
	sprite.owner = area

	# Create collision shape
	var shape = RectangleShape2D.new()
	shape.size = Vector2(32, 24)

	var collision = CollisionShape2D.new()
	collision.name = "CollisionShape2D"
	collision.position = Vector2(0, -4)
	collision.shape = shape
	area.add_child(collision, true)
	collision.owner = area

	# Save scene
	var packed_scene = PackedScene.new()
	var err = packed_scene.pack(area)
	if err != OK:
		print("  Error packing scene for %s: %s" % [dish_id, err])
		area.queue_free()
		return "error"

	err = ResourceSaver.save(packed_scene, prefab_path)
	if err != OK:
		print("  Error saving %s: %s" % [prefab_path, err])
		area.queue_free()
		return "error"

	area.queue_free()
	print("  Created %s.tscn" % dish_id)
	return "created"


func update_existing_prefab(prefab_path: String, dish_id: String, x: int, y: int) -> String:
	"""Update texture reference in existing prefab"""
	var packed_scene = load(prefab_path) as PackedScene
	if packed_scene == null:
		print("  Error: Failed to load %s" % prefab_path)
		return "error"

	var scene_state = packed_scene.get_state()
	var needs_update = false

	# Find the Sprite2D node and check its texture
	var dish_texture_path = "%s/%s.tres" % [DISH_TEXTURES_DIR, dish_id]

	for i in range(scene_state.get_node_count()):
		var node_type = scene_state.get_node_type(i)

		if node_type == "Sprite2D":
			# Check if texture needs updating
			for j in range(scene_state.get_node_property_count(i)):
				var prop_name = scene_state.get_node_property_name(i, j)
				if prop_name == "texture":
					var current_texture = scene_state.get_node_property_value(i, j)
					if current_texture != null:
						if current_texture.resource_path != dish_texture_path:
							needs_update = true
							break

	if not needs_update:
		print("  Skipped %s.tscn (up to date)" % dish_id)
		return "skipped"

	# Load the individual texture resource
	var dish_texture = load(dish_texture_path)
	if dish_texture == null:
		print("  Error: Failed to load texture resource %s" % dish_texture_path)
		return "error"

	# Instantiate, modify, and re-save
	var area = packed_scene.instantiate() as Area2D
	var sprite = area.get_node("Sprite2D") as Sprite2D

	if sprite != null:
		sprite.texture = dish_texture

		var new_packed_scene = PackedScene.new()
		var err = new_packed_scene.pack(area)
		if err == OK:
			err = ResourceSaver.save(new_packed_scene, prefab_path)
			if err == OK:
				area.queue_free()
				print("  Updated %s.tscn (texture reference)" % dish_id)
				return "updated"

	area.queue_free()
	print("  Error updating %s.tscn" % dish_id)
	return "error"
