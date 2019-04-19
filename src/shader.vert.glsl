precision highp float;

uniform mat4 u_model;
uniform mat4 u_view;

attribute vec2 a_position;
attribute vec2 a_center;

varying vec2 v_center;

void main() {
    v_center = (u_model * vec4(a_center, 1.0, 1.0)).xy;
    gl_Position = u_view * u_model * vec4(a_position, 0.0, 1.0);
}
