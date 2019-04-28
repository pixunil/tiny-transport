precision highp float;

uniform float u_size;
uniform mat4 u_modelView;

attribute vec2 a_position;

varying float v_size;

void main() {
    v_size = u_modelView[0].x * u_size;
    gl_PointSize = v_size;
    gl_Position = u_modelView * vec4(a_position, 0.0, 1.0);
}
