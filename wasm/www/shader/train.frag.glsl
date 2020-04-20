precision highp float;

uniform vec2 u_lineNamesShape;
uniform float u_lineNamesRatio;
uniform sampler2D u_lineNames;

varying vec3 v_color;
varying vec2 v_extent;
varying float v_lineNumber;
varying vec2 v_side;

void main() {
    vec2 padding = vec2(5.0, 10.0) / v_extent.yx;
    float lineNameWidth = 1.0 - 2.0 * padding.x;
    float lineNameHeight = lineNameWidth * u_lineNamesRatio * v_extent.y / v_extent.x;
    vec2 lineNameCoordinates = clamp((v_side - padding) / vec2(lineNameWidth, lineNameHeight), 0.0, 1.0);
    vec2 lineNameOffset = vec2(mod(v_lineNumber, u_lineNamesShape.x), floor(v_lineNumber / u_lineNamesShape.x));
    vec2 lineNameTextureCoordinates = (lineNameCoordinates + lineNameOffset) / u_lineNamesShape;
    vec4 lineNameDisplay = texture2D(u_lineNames, lineNameTextureCoordinates);
    gl_FragColor = vec4(mix(v_color, lineNameDisplay.rgb, lineNameDisplay.a), 1.0);
}
