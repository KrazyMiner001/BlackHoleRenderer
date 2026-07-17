alias num = f32;

struct Uniforms {
    M: num,
    a: num,
    camera_pos: vec3<num>,
    camera_size: vec2<num>,
    camera_normal: vec3<num>,
}

struct Variables {
    M: num,
    pos: vec4<num>,
    a: num,
};

const G = 1;
const c = 1;

const DELTA = 0.001;
const MAX_ITERATIONS = 50000;

const zeroMat = mat4x4<num>(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
const identityMat = mat4x4<num>(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1);

const maxIterColor = vec4(255, 0, 0, 255);

const skyRadius = 15;

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
@group(0) @binding(1)
var out: texture_storage_2d<rgba8uint, write>;

@compute @workgroup_size(8, 8)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let tex_size = vec2<num>(textureDimensions(out));
    let camera_relative = (vec2<num>(gid.xy) / tex_size) - 0.5;
    let pos = uniforms.camera_pos + vec3(camera_relative * uniforms.camera_size, 0);

    var variables: Variables;
    variables.M = uniforms.M;
    variables.a = uniforms.a;
    variables.pos = vec4(pos, 0);

    let photon_sphere = 1.5 * (1 + sqrt(1 - variables.a * variables.a));

    var velocity = vec4(uniforms.camera_normal, 0); //todo: better initial velocity

    var iter_count = 0;
    loop {
        iter_count++;
        if (iter_count >= MAX_ITERATIONS) {
            store_color(gid, maxIterColor);
            break;
        }

        variables.pos += velocity * DELTA;
        velocity += geodesic(velocity, variables) * DELTA;

        if (length(variables.pos.xyz) < photon_sphere) {
            store_color(gid, vec4(0, 255, 0, 255));
            break;
        }

        if (length(variables.pos.xyz) > skyRadius) {
            store_color(gid, vec4(vec3<u32>(variables.pos.xyz / 10 * 255), 255));
            break;
        }
    }
}

fn metric(vars: Variables) -> mat4x4<num> {
    const minkowski = mat4x4<num>(
        -1, 0, 0, 0,
        0, 1, 0, 0,
        0, 0, 0, 1,
        0, 0, 0, 1,
    );
    let r = calc_r(vars);
    let f = calc_f(vars, r);
    let k = calc_k(vars, r);

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

fn calc_f(vars: Variables, r: num) -> num {
    return (2 * G * vars.M * pow(r, 3)) / (pow(r, 4) + vars.a * vars.a * vars.pos.z * vars.pos.z);
}

fn calc_k(vars: Variables, r: num) -> vec4<num> {
    return vec4(
        1,
        (r * vars.pos.x + vars.a * vars.pos.y) / (r * r + vars.a * vars.a),
        (r * vars.pos.y - vars.a * vars.pos.x) / (r * r + vars.a * vars.a),
        vars.pos.z / r
    );
}

fn calc_r(vars: Variables) -> num {
    let minus_b = length(vars.pos) - vars.a * vars.a;

    return sqrt(
        0.5 * (minus_b + sqrt(minus_b * minus_b + 4 * vars.pos.z * vars.pos.z * vars.a * vars.a))
    );
}

fn inverse(mat: mat4x4<num>) -> mat4x4<num> {
    return (1 / determinant(mat)) *
        ((1/6) *
            (pow(tr(mat), 3) - 3 * tr(mat * tr(mat * mat)) + 2 * tr(mat * mat * mat)) * identityMat
            -0.5 * mat * (pow(tr(mat), 2) - tr(mat * mat)) + mat * mat * tr(mat) - mat * mat * mat
        );
}

fn christoffel(vars: Variables) -> array<mat4x4<num>, 4> {
    const epsilon = 0.00001;
    let g = metric(vars);

    let nabla0 = zeroMat;

    var varsClone = vars;
    varsClone.pos.x = vars.pos.x + sqrt(epsilon) * vars.pos.x;
    let nabla1 = (metric(varsClone) - g) * (1 / varsClone.pos.x - vars.pos.x);

    varsClone.pos.x = vars.pos.x;
    varsClone.pos.y = vars.pos.y + sqrt(epsilon) * vars.pos.y;
    let nabla2 = (metric(varsClone) - g) * (1 / varsClone.pos.y - vars.pos.y);

    varsClone.pos.y = vars.pos.y;
    varsClone.pos.z = vars.pos.z + sqrt(epsilon) * vars.pos.z;
    let nabla3 = (metric(varsClone) - g) * (1 / varsClone.pos.z - vars.pos.z);

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

fn geodesic(velocity: vec4<num>, vars: Variables) -> vec4<num> {
    var acceleration = vec4<num>(0);
    var christoffel_syms = christoffel(vars);

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