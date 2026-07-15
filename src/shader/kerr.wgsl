alias num = f32;

struct Uniforms {
    M: num,
    a: num,
}

struct Variables {
    M: num,
    x: num,
    y: num,
    z: num,
    a: num,
};

const G = 1;
const c = 1;

const zeroMat = mat4x4<num>(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@compute @workgroup_size(8, 8)
fn main() {

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
    return (2 * G * vars.M * pow(r, 3)) / (pow(r, 4) + vars.a * vars.a * vars.z * vars.z);
}

fn calc_k(vars: Variables, r: num) -> vec4<num> {
    return vec4(
        1,
        (r * vars.x + vars.a * vars.y) / (r * r + vars.a * vars.a),
        (r * vars.y - vars.a * vars.x) / (r * r + vars.a * vars.a),
        vars.z / r
    );
}

fn calc_r(vars: Variables) -> num {
    return sqrt(
        vars.a * (1 - vars.z * vars.z) / (vars.x * vars.x + vars.y * vars.y + vars.z * vars.z - 1)
    );
}

fn inverse(mat: mat4x4<num>) -> mat4x4<num> {
    return transpose(mat); //todo: actually take inverse, but this approximation should be fine for now
}

fn christoffel(vars: Variables) -> array<mat4x4<num>, 4> {
    const epsilon = 0.001;
    let g = metric(vars);

    let nabla0 = zeroMat;

    var varsClone = vars;
    varsClone.x = vars.x + sqrt(epsilon) * vars.x;
    let nabla1 = (metric(varsClone) - g) * (1 / varsClone.x - vars.x);

    varsClone.x = vars.x;
    varsClone.y = vars.y + sqrt(epsilon) * vars.y;
    let nabla2 = (metric(varsClone) - g) * (1 / varsClone.y - vars.y);

    varsClone.y = vars.y;
    varsClone.z = vars.z + sqrt(epsilon) * vars.z;
    let nabla3 = (metric(varsClone) - g) * (1 / varsClone.z - vars.z);

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