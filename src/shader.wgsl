struct Uniforms {
	aspect: f32;
};
[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

struct VertexInput {
	[[location(0)]] pos: vec3<f32>;
	[[location(1)]] clip: vec2<f32>;
};

struct VertexOutput {
	[[builtin(position)]] pos: vec4<f32>;
	[[location(0)]] clip: vec2<f32>;
};

[[stage(vertex)]]
fn vertex(input: VertexInput) -> VertexOutput {
	var output: VertexOutput;
	if (uniforms.aspect > 1.0) {
		output.pos = vec4<f32>(input.pos, 1.0) / vec4<f32>(1.0, uniforms.aspect, 1.0, 1.0);
	} else {
		output.pos = vec4<f32>(input.pos, 1.0) * vec4<f32>(uniforms.aspect, 1.0, 1.0, 1.0);
	}
	output.clip = input.clip;
	return output;
}

[[stage(fragment)]]
fn fragment(input: VertexOutput) -> [[location(0)]] vec4<f32> {
	return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}