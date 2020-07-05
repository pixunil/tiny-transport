#version 300 es

uniform float u_scaling;
uniform mat4 u_modelView;

in vec2 v_position;
in lowp uint v_type;

flat out lowp uint f_type;
out float f_size;

void main() {
    f_type = v_type;
    if (f_type <= 1U) {
        f_size = u_scaling * 60.0;
    } else if (f_type == 2U) {
        f_size = u_scaling * 75.0;
    } else {
        f_size = u_scaling * 90.0;
    }
    gl_PointSize = f_size;
    gl_Position = u_modelView * vec4(v_position, 0.0, 1.0);
}
