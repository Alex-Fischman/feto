extends Spatial

# TODO: test

func on_damaged(_damage):
	get_child(0).set_process(true)
