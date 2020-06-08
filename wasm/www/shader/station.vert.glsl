precision highp float;

uniform float u_scaling;
uniform mat4 u_modelView;

attribute vec2 a_position;
attribute float a_type;

varying float v_type;
varying float v_size;

void main() {
    v_type = a_type;
    if (v_type <= 1.0) {
        v_size = u_scaling * 60.0;
    } else if (v_type == 2.0) {
        v_size = u_scaling * 75.0;
    } else {
        v_size = u_scaling * 90.0;
    }
    gl_PointSize = v_size;
    gl_Position = u_modelView * vec4(a_position, 0.0, 1.0);
}
