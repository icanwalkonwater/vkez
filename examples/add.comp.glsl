#version 450

layout(set = 0, binding = 0) readonly buffer A {
    float a[];
};
layout(set = 0, binding = 1) readonly buffer B {
    float b[];
};
layout(set = 0, binding = 2) buffer C {
    float c[];
};

layout (local_size_x = 256, local_size_y = 1, local_size_z = 1) in;

void main() {
    uint index = gl_GlobalInvocationID.x;

    c[index] = (a[index] + b[index] + index) * 3.1415;
}
