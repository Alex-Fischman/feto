extends KinematicBody

signal spell_update(spell)

export var MODIFIER_AIR : float
export var MODIFIER_FIRE : float
export var MODIFIER_ACID : float
export var MODIFIER_PRESSURE_SPACE : float
export var MODIFIER_PRESSURE_STRENGTH : float
export var MODIFIER_SHOCK : float
export var MODIFIER_RADIANCE : float
export var MODIFIER_LIFE : float
export var INITIAL_TRAVEL_TIME : float
export var INITIAL_TRAVEL_DIST : float
export var INITIAL_OFFSET_DIST : float

export var SPEED : float
export var SPEED_LERP : float
export var MOUSE : float
export var GRAVITY : float

export var YEET : PackedScene
export var HEAT : PackedScene
export var TRAP : PackedScene

var last := 0
var push := false
var spell := []
func handle_symbol(symbol):
	if push:
		spell.push_back(last | symbol)
		emit_signal("spell_update", spell)
	push = !push
	last = symbol

func _input(event):
	if event.is_action_pressed("pause"):
		if Input.get_mouse_mode() == Input.MOUSE_MODE_CAPTURED:
			  Input.set_mouse_mode(Input.MOUSE_MODE_VISIBLE)
		else: Input.set_mouse_mode(Input.MOUSE_MODE_CAPTURED)
	
	if Input.get_mouse_mode() == Input.MOUSE_MODE_CAPTURED:
		if event is InputEventMouseMotion:
			rotation.y += -MOUSE * event.relative.x
			$Camera.rotation.x += -MOUSE * event.relative.y
			$Camera.rotation.x = clamp($Camera.rotation.x, -PI / 2, PI / 2)
		
		if event.is_action_pressed("earth"):
			handle_symbol(Symbol.Earth)
		if event.is_action_pressed("water"):
			handle_symbol(Symbol.Water)
		if event.is_action_pressed("air"):
			handle_symbol(Symbol.Air)
		if event.is_action_pressed("fire"):
			handle_symbol(Symbol.Fire)
		if event.is_action_pressed("repeat"):
			handle_symbol(last)
		if event.is_action_pressed("cast"):
			if spell.empty(): return

			var strength = 1
			var space = 1
			var time = 1
			var travel_time = INITIAL_TRAVEL_TIME
			var travel_dist = INITIAL_TRAVEL_DIST

			var is_trap = false
			var initial_offset = Vector3.FORWARD * INITIAL_OFFSET_DIST
			
			for s in spell.slice(1, len(spell)):
				if s == Symbol.Earth:
					is_trap = true
				elif s == Symbol.Water:
					travel_dist = 0
					initial_offset = Vector3.ZERO
				elif s == Symbol.Air:
					travel_dist *= MODIFIER_AIR
				elif s == Symbol.Fire:
					travel_time /= MODIFIER_FIRE
				elif s == Symbol.Acid:
					time *= MODIFIER_ACID
				elif s == Symbol.Pressure:
					space /= MODIFIER_PRESSURE_SPACE
					strength *= MODIFIER_PRESSURE_STRENGTH
				elif s == Symbol.Shock:
					time /= MODIFIER_SHOCK
				elif s == Symbol.Radiance:
					space *= MODIFIER_RADIANCE
				elif s == Symbol.Life:
					strength *= MODIFIER_LIFE

			var parent = get_tree().root.get_child(0)
			if is_trap:
				var trap = TRAP.instance()
				parent.add_child(trap)
				parent = trap

			if spell[0] == Symbol.Earth:
				if spell.count(Symbol.Void) % 2 == 0: pass # TODO
				else: pass # TODO
			elif spell[0] == Symbol.Water:
				if spell.count(Symbol.Void) % 2 == 0: pass # TODO
				else: pass # TODO
			elif spell[0] == Symbol.Air:
				if spell.count(Symbol.Void) % 2 == 0:
					# pass # TODO: use parameters
					var node = YEET.instance()
					parent.add_child(node)
					node.global_transform = $Camera.global_transform
					node.translate(initial_offset)
					node.rotate_object_local(Vector3.UP, PI)

					node.travel_time = travel_time
					node.effect_time = time
					node.effect_strength = strength
					node.scale_object_local(Vector3.ONE * space)
					node.vel = vel - node.global_transform.basis.z * travel_dist / travel_time
				else: pass # TODO
			elif spell[0] == Symbol.Fire:
				if spell.count(Symbol.Void) % 2 == 0:
					var node = HEAT.instance()
					parent.add_child(node)
					node.global_transform = $Camera.global_transform
					node.translate(initial_offset)

					node.travel_time = travel_time
					node.effect_time = time
					node.dps_mult = strength / time
					node.scale_object_local(Vector3.ONE * space)
					node.vel = vel - node.global_transform.basis.z * travel_dist / travel_time
				else: pass # TODO
			elif spell[0] == Symbol.Acid:
				if spell.count(Symbol.Void) % 2 == 0: pass # TODO
				else: pass # TODO
			elif spell[0] == Symbol.Pressure:
				if spell.count(Symbol.Void) % 2 == 0: pass # TODO
				else: pass # TODO
			elif spell[0] == Symbol.Shock:
				if spell.count(Symbol.Void) % 2 == 0: pass # TODO
				else: pass # TODO
			elif spell[0] == Symbol.Radiance:
				if spell.count(Symbol.Void) % 2 == 0: pass # TODO
				else: pass # TODO
			elif spell[0] == Symbol.Life:
				if spell.count(Symbol.Void) % 2 == 0: pass # TODO
				else: pass # TODO
			elif spell[0] == Symbol.Void:
				if spell.count(Symbol.Void) % 2 == 1: pass # TODO
				else: pass # TODO

			if is_trap:
				parent.get_child(0).set_process(false)

			spell = []
			last = 0
			push = false
			emit_signal("spell_update", spell)

var vel = Vector3.ZERO
func _physics_process(_delta):
	var target = SPEED * Vector3(
		Input.get_action_strength("right") - Input.get_action_strength("left"),
		0,
		Input.get_action_strength("back") - Input.get_action_strength("forward")
	).normalized().rotated(Vector3.UP, rotation.y)
	
	vel.y -= GRAVITY
	if is_on_floor(): vel = vel.linear_interpolate(target, SPEED_LERP)
	vel = move_and_slide(vel, Vector3.UP)
