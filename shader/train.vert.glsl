#version 300 es

uniform mat4 u_modelView;

in vec2 v_position;
in vec3 v_color;
in vec2 v_extent;
in mediump uint v_lineNumber;
in vec2 v_side;

flat out vec3 f_color;
out vec2 f_extent;
flat out mediump uint f_lineNumber;
out vec2 f_side;

void main() {
    gl_Position = u_modelView * vec4(v_position, 0.0, 1.0);
    f_color = v_color;
    f_extent = v_extent;
    f_lineNumber = v_lineNumber;
    f_side = v_side;
}
