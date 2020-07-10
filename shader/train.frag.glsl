#version 300 es

precision mediump float;
precision highp sampler2D;

uniform uvec2 u_lineNamesShape;
uniform float u_lineNamesRatio;
uniform sampler2D u_lineNames;

flat in vec3 f_color;
in vec2 f_extent;
flat in mediump uint f_lineNumber;
in vec2 f_side;

out vec4 o_color;

void main() {
    vec2 padding = vec2(5.0, 10.0) / f_extent.yx;
    float lineNameWidth = 1.0 - 2.0 * padding.x;
    float lineNameHeight = lineNameWidth * u_lineNamesRatio * f_extent.y / f_extent.x;
    vec2 lineNameCoordinates = clamp((f_side - padding) / vec2(lineNameWidth, lineNameHeight), 0.0, 1.0);
    vec2 lineNameOffset = vec2(f_lineNumber % u_lineNamesShape.x, f_lineNumber / u_lineNamesShape.x);
    vec2 lineNameTextureCoordinates = (lineNameCoordinates + lineNameOffset) / vec2(u_lineNamesShape.xy);
    vec4 lineNameDisplay = texture(u_lineNames, lineNameTextureCoordinates);
    o_color = vec4(mix(f_color, lineNameDisplay.rgb, lineNameDisplay.a), 1.0);
}
