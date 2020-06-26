precision highp float;

uniform mat4 u_modelView;

attribute vec2 a_position;
attribute vec3 a_color;
attribute vec2 a_extent;
attribute float a_lineNumber;
attribute vec2 a_side;

varying vec3 v_color;
varying vec2 v_extent;
varying float v_lineNumber;
varying vec2 v_side;

void main() {
    gl_Position = u_modelView * vec4(a_position, 0.0, 1.0);
    v_color = a_color;
    v_extent = a_extent;
    v_lineNumber = a_lineNumber;
    v_side = a_side;
}
