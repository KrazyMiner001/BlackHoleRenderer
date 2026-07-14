struct Uniforms {
    position: vec3<f32>,
    radius: f32,
    camera_size: vec2<f32>,
    camera_normal: vec3<f32>,
}

const max_iterations = 10000;
const delta = 0.1;

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
@group(0) @binding(1)
var out_data: texture_storage_2d<rgba8uint, write>;

@compute @workgroup_size(8, 8)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    var tex_size = vec2<f32>(textureDimensions(out_data));
    var camera_relative = (vec2<f32>(gid.xy) / tex_size) - 0.5;
    var pos = uniforms.position + vec3(camera_relative * uniforms.camera_size, 0);

    var iter_count = 0;
    loop {
        iter_count += 1;
        if (iter_count > max_iterations) {
            textureStore(out_data, gid.xy, vec4(0, 0, 0, 255));
            break;
        }

        if (length(pos) > 0.95 && length(pos) < 1.05) {
            textureStore(out_data, gid.xy, vec4(255, 255, 255, 255));
            break;
        }
        if (length(pos) < 0.95) {
            textureStore(out_data, gid.xy, vec4(255, 0, 127, 255));
            break;
        }
        pos += uniforms.camera_normal * delta;
    };
}