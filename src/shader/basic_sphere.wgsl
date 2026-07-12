struct Uniforms {
    position: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
@group(0) @binding(1)
var out_data: texture_storage_2d<rgba8uint, write>;

@compute @workgroup_size(8, 8)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    textureStore(out_data, gid.xy, vec4(gid.x, gid.y, 100, 256));
}