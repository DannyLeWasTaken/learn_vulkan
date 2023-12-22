#version 450

layout(location = 0) out vec4 outColor;

layout(location = 0) in vec3 frag_color;

void main() {
    outColor = vec4(frag_color, 1.0);
    // hi  A A
}