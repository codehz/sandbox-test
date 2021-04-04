#version 450

layout(location = 0) in uint id;

void main() {
  gl_Position = vec4(vec2(float(id % 2), float(id / 2)) * 2.0 - 1.0, 0.0, 1.0);
}