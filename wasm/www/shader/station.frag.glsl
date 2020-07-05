#version 300 es

precision mediump float;

flat in lowp uint f_type;
in float f_size;

out vec4 o_color;

const float margin = 2.0;

const float interchangeInnerFraction = 0.35;
const float interchangeOuterFraction = 0.5;

const vec3 busColor = vec3(0.49, 0.09, 0.42);
const vec3 tramColor = vec3(0.8, 0.04, 0.13);
const vec3 ferryColor = vec3(0.0, 0.5, 0.72);
const vec3 interchangeInner = vec3(1.0, 1.0, 1.0);
const vec3 interchangeOuter = vec3(0.0, 0.0, 0.1);

void main() {
    float radius = (f_size + margin) * length(gl_PointCoord - 0.5);
    vec3 color;
    float alpha = 1.0;
    if (f_type == 0U) {
        color = busColor;
        alpha = 1.0 - (radius - f_size * 0.5);
    } else if (f_type == 1U) {
        color = tramColor;
    } else if (f_type == 2U) {
        color = ferryColor;
        alpha = 1.0 - (radius - f_size * 0.5);
    } else {
        float innerRadius = f_size * interchangeInnerFraction;
        float outerRadius = f_size * interchangeOuterFraction;

        color = mix(interchangeInner, interchangeOuter, radius - innerRadius);
        alpha = 1.0 - (radius - outerRadius);
    }
    o_color = vec4(color, alpha);
}
