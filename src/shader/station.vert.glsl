precision highp float;

uniform float u_size;
uniform mat4 u_modelView;

attribute vec2 a_position;

void main() {
    gl_PointSize = u_size;
    gl_Position = u_modelView * vec4(a_position, 0.0, 1.0);
}
