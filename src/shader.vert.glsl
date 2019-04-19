precision highp float;

uniform float u_size;
uniform mat4 u_model;
uniform mat4 u_view;

attribute vec2 a_position;

void main() {
    gl_PointSize = u_size;
    gl_Position = u_view * u_model * vec4(a_position, 0.0, 1.0);
}
