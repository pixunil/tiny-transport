precision highp float;

varying float v_size;

const float margin = 2.0;

const float innerFraction = 0.35;
const float borderFraction = 0.5;

const vec3 innerColor = vec3(1.0, 1.0, 1.0);
const vec3 borderColor = vec3(0.0, 0.0, 0.1);


void main() {
    float radius = (v_size + margin) * length(gl_PointCoord - 0.5);
    float innerRadius = v_size * innerFraction;
    float borderRadius = v_size * borderFraction;

    vec3 color = mix(innerColor, borderColor, radius - innerRadius);
    float alpha = 1.0 - (radius - borderRadius);
    gl_FragColor = vec4(color, alpha);
}
