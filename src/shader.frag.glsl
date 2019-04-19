precision highp float;

varying vec2 v_center;

const vec3 innerColor = vec3(1.0, 1.0, 1.0);
const vec3 borderColor = vec3(0.0, 0.0, 0.1);

const float innerSize = 6.0;
const float borderSize = 9.0;

void main() {
    float radius = length(gl_FragCoord.xy - v_center);

    vec3 color = mix(innerColor, borderColor, radius - innerSize);
    float alpha = 1.0 - (radius - borderSize);
    gl_FragColor = vec4(color, alpha);
}
