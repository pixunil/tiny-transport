#version 300 es

uniform mat4 u_modelView;

in vec2 v_position;

void main() {
    gl_Position = u_modelView * vec4(v_position, 0.0, 1.0);
}
