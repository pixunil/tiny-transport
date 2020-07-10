#version 300 es

precision mediump float;

uniform float u_size;
uniform float u_minuteAngle;
uniform float u_hourAngle;

in vec2 f_position;

out vec4 o_color;

const vec3 fillColor = vec3(1.0, 1.0, 0.98);
const vec3 strokeColor = vec3(0.0, 0.0, 0.1);

float line(float angle, float width, float middle, float halvedLength) {
    mat2 transformation = mat2(
        cos(-angle), sin(-angle),
        -sin(-angle), cos(-angle)
    );
    vec2 transformed = transformation * f_position;
    return clamp(u_size * (width - abs(transformed.y)), 0.0, 1.0)
        * clamp(u_size * (halvedLength - abs(transformed.x - middle)), 0.0, 1.0);
}

void main() {
    float distanceToCenter = u_size * length(f_position);
    float angle = atan(f_position.y, f_position.x);
    float nearestTickAngle = round(angle / radians(30.0)) * radians(30.0);

    float isStroke = clamp(distanceToCenter - u_size * 0.85, 0.0, 1.0);
    isStroke = max(isStroke, line(nearestTickAngle, 0.02, 0.7, 0.1));
    isStroke = max(isStroke, line(u_minuteAngle, 0.03, 0.3, 0.4));
    isStroke = max(isStroke, line(u_hourAngle, 0.03, 0.2, 0.3));
    vec3 color = mix(fillColor, strokeColor, isStroke);
    float alpha = clamp(1.0 - (distanceToCenter - u_size * 0.95), 0.0, 1.0);
    o_color = vec4(color, alpha);
}
