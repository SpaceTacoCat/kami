struct Globals {
    transform: mat4x4<f32>;
};

struct VertexInput {
    [[builtin(vertex_index)]] vertex_index: u32;
    [[location(0)]] aabb: vec4<f32>; // left top right bottom
    [[location(1)]] z_pos: f32;
    [[location(2)]] color: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(1)]] f_color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    var pos: vec2<f32> = vec2<f32>(0.0, 0.0);

    switch (i32(input.vertex_index)) {
        case 0: {
            pos = vec2<f32>(input.aabb.x, input.aabb.y);
        }
        case 1: {
            pos = vec2<f32>(input.aabb.z, input.aabb.y);
        }
        case 2: {
            pos = vec2<f32>(input.aabb.x, input.aabb.w);
        }
        case 3: {
            pos = vec2<f32>(input.aabb.z, input.aabb.w);
        }
        default: {}
    }

    out.f_color = vec4<f32>(input.color, 1.0);
    out.position = vec4<f32>(pos, input.z_pos, 1.0);

    return out;
}

[[stage(fragment)]]
fn fs_main(input: VertexOutput) -> [[location(0)]] vec4<f32> {
    return input.f_color;
}
