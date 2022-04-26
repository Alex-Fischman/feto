extends MeshInstance

export var DISTANCE : float
export var SCALING : float
export var WIDTH : float
export var OUTER : float
export var LERP : float

export var color_map : Dictionary = {
	Symbol.Earth:    Color.orange,
	Symbol.Water:    Color.blue,
	Symbol.Air:      Color.cyan,
	Symbol.Fire:     Color.red,
	Symbol.Acid:     Color.green,
	Symbol.Pressure: Color.cornflower,
	Symbol.Shock:    Color.yellow,
	Symbol.Radiance: Color.purple,
	Symbol.Life:     Color.white,
	Symbol.Void:     Color.black,
}

func _physics_process(_delta):
	scale = scale.linear_interpolate(Vector3.ONE * pow(SCALING, length), LERP)

var length = 0
func _on_spell_update(spell):
	length = len(spell)
	for _i in mesh.get_surface_count(): mesh.surface_remove(0)
	for i in len(spell):
		var symbol = spell[i]
		var color = color_map.get(symbol)
		
		if symbol & Symbol.Earth != 0: generate_surface(4, color, i, PI / 4)
		if symbol & Symbol.Water != 0: generate_surface(32, color, i)
		if symbol & Symbol.Air   != 0: generate_surface(4, color, i)
		
		if symbol == Symbol.Void:
			var scaling = pow(SCALING, -i)
			var outer = sqrt(pow(OUTER, 2) / 2)
			var a = outer - WIDTH * sqrt(5)
			var b = outer - WIDTH * (1 + sqrt(5)) / 2
			var verts = PoolVector3Array([
				Vector3(0, a, 0) * scaling,
				Vector3(0, outer, 0) * scaling,
				Vector3(b, -outer + WIDTH, 0) * scaling,
				Vector3(outer, -outer, 0) * scaling,
				Vector3(-b, -outer + WIDTH, 0) * scaling,
				Vector3(-outer, -outer, 0) * scaling,
				Vector3(0, a, 0) * scaling,
				Vector3(0, outer, 0) * scaling
			])
			
			var colors = PoolColorArray()
			for _j in 8: colors.push_back(color)
			
			var arr = []
			arr.resize(Mesh.ARRAY_MAX)
			arr[Mesh.ARRAY_VERTEX] = verts
			arr[Mesh.ARRAY_COLOR] = colors
			mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLE_STRIP, arr)
		elif symbol & Symbol.Fire != 0:
			generate_surface(3, color, i)

func generate_surface(sides, color, index, angle_offset = 0):
	var inner = (OUTER - WIDTH / cos(PI / sides)) * pow(SCALING, -index)
	var outer =  OUTER                            * pow(SCALING, -index)
	
	var verts = PoolVector3Array()
	var vec = Vector3.UP.rotated(Vector3.FORWARD, angle_offset)
	verts.push_back(vec * inner)
	verts.push_back(vec * outer)
	for _i in sides:
		vec = vec.rotated(Vector3.FORWARD, 2 * PI / sides)
		verts.push_back(vec * inner)
		verts.push_back(vec * outer)
	
	var colors = PoolColorArray()
	for _j in sides * 2 + 2: colors.push_back(color)
	
	var arr = []
	arr.resize(Mesh.ARRAY_MAX)
	arr[Mesh.ARRAY_VERTEX] = verts
	arr[Mesh.ARRAY_COLOR] = colors
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLE_STRIP, arr)
