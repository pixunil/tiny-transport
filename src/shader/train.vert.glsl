precision highp float;

uniform mat4 u_modelView;

attribute vec2 a_position;
attribute vec3 a_color;

varying vec3 v_color;

void main() {
    gl_Position = u_modelView * vec4(a_position, 0.0, 1.0);
    v_color = a_color;
}
