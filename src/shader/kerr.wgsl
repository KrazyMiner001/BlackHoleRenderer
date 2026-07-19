alias num = f32;

struct Uniforms {
    M: num,
    a: num,
    camera_pos: vec3<num>,
    camera_size: vec2<num>,
    camera_normal: vec3<num>,
}

const G = 1;
const c = 1;

const DELTA = 0.025;
const MAX_ITERATIONS = 10000;

const zeroMat = mat4x4<num>();
const identityMat = mat4x4<num>(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1);
const minkowski = mat4x4<num>(
        -1, 0, 0, 0,
        0, 1, 0, 0,
        0, 0, 0, 1,
        0, 0, 0, 1,
    );

const maxIterColor = vec4(255, 0, 0, 255);

const skyRadius = 15;

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
@group(0) @binding(1)
var out: texture_storage_2d<rgba8uint, write>;
@group(1) @binding(2)
var skymap: texture_2d<u32>;
@group(1) @binding(3)
var skymap_sampler: sampler;

@compute @workgroup_size(8, 8)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let tex_size = vec2<num>(textureDimensions(out));
    let camera_relative = (vec2<num>(gid.xy) / tex_size) - 0.5;
    let initial_pos = uniforms.camera_pos + vec3(camera_relative * uniforms.camera_size, 0);

    var pos = vec4(initial_pos, 0);

    let photon_sphere = 1.5 * (1 + sqrt(1 - uniforms.a * uniforms.a));

    var velocity = vec4(uniforms.camera_normal, 0); //todo: better initial velocity

    var iter_count = 0;
    loop {
        iter_count++;
        if (iter_count >= MAX_ITERATIONS) {
            store_color(gid, maxIterColor);
            break;
        }

        { //Runge-Kutta
            let k1pos = velocity;
            let k1vel = geodesic(velocity, pos);

            let k2pos = velocity + k1vel * (DELTA / 2);
            let k2vel = geodesic(velocity + k1vel * (DELTA / 2), pos + k1pos * (DELTA / 2));

            let k3pos = velocity + k2vel * (DELTA / 2);
            let k3vel = geodesic(velocity + k2vel * (DELTA / 2), pos + k2pos * (DELTA / 2));

            let k4pos = velocity + k3vel * DELTA;
            let k4vel = geodesic(velocity + k3vel * DELTA, pos + k3pos * DELTA);

            velocity += (DELTA / 6) * (k1vel + k2vel + k3vel + k4vel);
            pos += (DELTA / 6) * (k1pos + k2pos + k3pos + k4pos);
        }

        if (length(pos.xyz) < photon_sphere) {
            store_color(gid, vec4(0, 255, 0, 255));
            break;
        }

        if (length(pos.xyz) > skyRadius) {
            store_color(gid, vec4(vec3<u32>(pos.xyz / 10 * 255), 255));
            break;
        }
    }
}

fn metric(pos: vec4<num>) -> mat4x4<num> {
    let r = calc_r(pos);
    let f = calc_f(pos, r);
    let k = calc_k(pos, r);

    return minkowski + f * outer_product(k, k);
}

fn outer_product(first: vec4<num>, second: vec4<num>) -> mat4x4<num> {
    return mat4x4(
        first[0] * second,
        first[1] * second,
        first[2] * second,
        first[3] * second,
    );
}

fn calc_f(pos: vec4<num>, r: num) -> num {
    return (2 * G * uniforms.M * pow(r, 3)) / (pow(r, 4) + uniforms.a * uniforms.a * pos.z * pos.z);
}

fn calc_k(pos: vec4<num>, r: num) -> vec4<num> {
    return vec4(
        1,
        (r * pos.x + uniforms.a * pos.y) / (r * r + uniforms.a * uniforms.a),
        (r * pos.y - uniforms.a * pos.x) / (r * r + uniforms.a * uniforms.a),
        pos.z / r
    );
}

fn calc_r(pos: vec4<num>) -> num {
    let minus_b = length(pos.xyz) - uniforms.a * uniforms.a;

    return sqrt(
        0.5 * (minus_b + sqrt(minus_b * minus_b + 4 * pos.z * pos.z * uniforms.a * uniforms.a))
    );
}

fn inverse(mat: mat4x4<num>) -> mat4x4<num> {
    return (1 / determinant(mat)) *
        ((1/6) *
            (pow(tr(mat), 3) - 3 * tr(mat * tr(mat * mat)) + 2 * tr(mat * mat * mat)) * identityMat
            -0.5 * mat * (pow(tr(mat), 2) - tr(mat * mat)) + mat * mat * tr(mat) - mat * mat * mat
        );
}

fn christoffel(pos: vec4<num>) -> array<mat4x4<num>, 4> {
    const epsilon = 0.00001;
    let g = metric(pos);

    let nabla0 = zeroMat;

    let pos_deltax = pos + vec4(sqrt(epsilon) * pos.x, 0, 0, 0);
    let nabla1 = (metric(pos_deltax) - g) * (1 / pos_deltax.x - pos.x);

    let pos_deltay = pos + vec4(0, sqrt(epsilon) * pos.y, 0, 0);
    let nabla2 = (metric(pos_deltay) - g) * (1 / pos_deltay.y - pos.y);

    let pos_deltaz = pos + vec4(0, 0, sqrt(epsilon) * pos.z, 0);
    let nabla3 = (metric(pos_deltaz) - g) * (1 / pos_deltaz.z - pos.y);

    let nabla = array(
        nabla0,
        nabla1,
        nabla2,
        nabla3,
    );

    let inv_g = inverse(g);

    var symbol = array<mat4x4<num>, 4>(zeroMat, zeroMat, zeroMat, zeroMat);

    for (var i = 0; i < 4; i++) {
        for (var k = 0; i < 4; i++) {
            for (var l = 0; i < 4; i++) {
                var total: num = 0;
                for (var m = 0; m < 4; m++) {
                    total += 0.5 * inv_g[i][m] * (nabla[l][m][k] + nabla[k][m][l] - nabla[m][k][l]);
                }
                symbol[i][k][l] = total;
            }
        }
    }

    return symbol;
}

fn geodesic(velocity: vec4<num>, pos: vec4<num>) -> vec4<num> {
    var acceleration = vec4<num>(0);
    var christoffel_syms = christoffel(pos);

    for (var mu = 0; mu < 4; mu++) {
        var total: num = 0;
        for (var alpha = 0; alpha < 4; alpha++) {
            for (var beta = 0; beta < 4; beta++) {
                total += velocity[alpha] * velocity[beta] * christoffel_syms[mu][alpha][beta];
            }
        }
        acceleration[mu] = -total;
    }

    return acceleration;
}

fn store_color(gid: vec3<u32>, color: vec4<u32>) {
    textureStore(out, gid.xy, color);
}

fn tr(matrix: mat4x4<num>) -> num {
    return matrix[0][0] + matrix[1][1] + matrix[2][2] + matrix[3][3];
}