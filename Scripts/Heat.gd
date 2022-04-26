extends Area

# TODO: add explosion effect
# TODO: add damage signals

export var BASE_DPS : float

var time = 0
var active = true

var vel : Vector3
var travel_time : float
var effect_time : float
var dps_mult : float

func _process(delta):
	if time > travel_time + $Fire.lifetime: queue_free()
	elif time > travel_time and active:
		active = false
		$Light.visible = false
		$Fire.emitting = false
	time += delta

func _physics_process(delta):
	if active: global_translate(vel * delta)
