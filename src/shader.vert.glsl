precision highp float;

attribute vec2 a_position;

varying vec4 v_position;

void main() {
    v_position = vec4(a_position, 0.0, 1.0);
    gl_Position = v_position;
}
