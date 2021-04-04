#version 450

#define QUALITY 32

layout(points) in;
layout(triangle_strip, max_vertices = QUALITY * 2 + 1) out;

layout(location = 0) in vec3 gcolor[];
layout(location = 1) in float gradius[];
layout(location = 0) out vec3 v_color;

layout(location = 2) uniform float aspect_ratio;

// const float PI = 3.1415926;
const float Tau = 6.28318530718;
const float Delta = Tau / QUALITY;

void main() {
  v_color = gcolor[0];
  vec4 center = gl_in[0].gl_Position;
  for (float i = -Delta; i < Tau; i += Delta) {
    vec4 off = vec4(cos(i) / aspect_ratio, sin(i), 0.0, 0.0) * gradius[0];
    gl_Position = center + off;
    EmitVertex();
    gl_Position = center;
    EmitVertex();
  }
  EndPrimitive();
}
