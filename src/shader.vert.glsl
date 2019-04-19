precision highp float;

uniform mat4 u_model;

attribute vec2 a_position;
attribute vec2 a_center;

varying vec2 v_center;

void main() {
    v_center = a_center;
    gl_Position = u_model * vec4(a_position, 0.0, 1.0);
}
