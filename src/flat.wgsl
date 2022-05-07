struct VertexInput {
	[[location(0)]] pos: vec2<f32>;
	[[location(1)]] color: vec3<f32>;
};

struct VertexOutput {
	[[builtin(position)]] pos: vec4<f32>;
	[[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(input: VertexInput) -> VertexOutput {
	var output: VertexOutput;
	output.pos = vec4<f32>(input.pos, 0.0, 1.0);
	output.color = vec4<f32>(input.color, 1.0);
	return output;
}

[[stage(fragment)]]
fn fragment(input: VertexOutput) -> [[location(0)]] vec4<f32> {
	return input.color;
}
