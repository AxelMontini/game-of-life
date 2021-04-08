#version 140
uniform mat4 matrix;
in vec2 position;
out vec2 vCoords;
void main() {
    gl_Position = matrix * vec4(position, 0.0, 1.0);
    vCoords = position;
}