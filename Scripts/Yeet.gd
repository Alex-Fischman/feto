extends Particles

const BASE_FORCE = 20

var time = 0
var active = true

var vel : Vector3
var travel_time : float
var effect_time : float
var effect_strength : float

func _process(delta):
	if time > travel_time + lifetime: queue_free()
	elif time > travel_time and active:
		active = false
		$Area.collision_mask = 0
		emitting = false
	time += delta
	
	if active:
		for body in $Area.get_overlapping_areas() + $Area.get_overlapping_bodies():
			if not body.get("vel") == null:
				body.vel = body.vel.linear_interpolate(global_transform.basis.z * 
					BASE_FORCE * effect_strength, 0.5)

func _physics_process(delta):
	if active: global_translate(-vel * delta)
