@group(0) @binding(0)
var<storage, read> in_data: array<u32>;
@group(0) @binding(1)
var<storage, read_write> out_data: array<u32>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    out_data[0] = in_data[0] * 2;
}