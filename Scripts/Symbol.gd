class_name Symbol

enum {
	Earth = 0b0001,
	Water = 0b0010,
	Air   = 0b0100,
	Fire  = 0b1000,
	Acid     = Water | Fire,
	Pressure = Earth | Air,
	Shock    = Air   | Fire,
	Radiance = Water | Air,
	Life     = Earth | Water,
	Void     = Earth | Fire
}
