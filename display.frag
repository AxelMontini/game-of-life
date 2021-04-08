#version 140
uniform sampler2D grid;
uniform float scale;
in vec2 vCoords;
out vec4 f_color;
void main() {
    f_color = texture(grid, vCoords / scale);
}