#version 300 es

in vec2 v_position;

out vec2 f_position;

void main() {
    f_position = v_position;
    gl_Position = vec4(v_position, 0.0, 1.0);
}
