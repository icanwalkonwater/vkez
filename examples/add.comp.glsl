#version 450

layout(set = 0, binding = 0) readonly buffer A {
    float a[];
} aa;
layout(set = 0, binding = 1) readonly buffer B {
    float b[];
} bb;
layout(set = 0, binding = 2) buffer C {
    float c[];
};

layout (local_size_x = 256) in;

void main() {
    uint index = gl_GlobalInvocationID.x;

    c[index] = (aa.a[index] + bb.b[index] + index) * 3.1415;
}
