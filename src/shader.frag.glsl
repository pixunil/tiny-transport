precision highp float;

uniform float u_size;

const vec3 innerColor = vec3(1.0, 1.0, 1.0);
const vec3 borderColor = vec3(0.0, 0.0, 0.1);

const float innerSize = 6.0;
const float borderSize = 9.0;

void main() {
    float radius = u_size * length(gl_PointCoord - 0.5);

    vec3 color = mix(innerColor, borderColor, radius - innerSize);
    float alpha = 1.0 - (radius - borderSize);
    gl_FragColor = vec4(color, alpha);
}
