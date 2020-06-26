precision highp float;

varying float v_type;
varying float v_size;

const float margin = 2.0;

const float interchangeInnerFraction = 0.35;
const float interchangeOuterFraction = 0.5;

const vec3 busColor = vec3(0.49, 0.09, 0.42);
const vec3 tramColor = vec3(0.8, 0.04, 0.13);
const vec3 ferryColor = vec3(0.0, 0.5, 0.72);
const vec3 interchangeInner = vec3(1.0, 1.0, 1.0);
const vec3 interchangeOuter = vec3(0.0, 0.0, 0.1);


void main() {
    float radius = (v_size + margin) * length(gl_PointCoord - 0.5);
    vec3 color;
    float alpha = 1.0;
    if (v_type == 0.0) {
        color = busColor;
        alpha = 1.0 - (radius - v_size * 0.5);
    } else if (v_type == 1.0) {
        color = tramColor;
    } else if (v_type == 2.0) {
        color = ferryColor;
        alpha = 1.0 - (radius - v_size * 0.5);
    } else {
        float innerRadius = v_size * interchangeInnerFraction;
        float outerRadius = v_size * interchangeOuterFraction;

        color = mix(interchangeInner, interchangeOuter, radius - innerRadius);
        alpha = 1.0 - (radius - outerRadius);
    }
    gl_FragColor = vec4(color, alpha);
}
