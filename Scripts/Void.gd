extends OmniLight

export var AMPLITUDE = 0.1;
export var FREQUENCY = 2;

var time = 0

func _process(delta):
	time += delta
	light_energy = AMPLITUDE * sin(time * FREQUENCY) + 1
