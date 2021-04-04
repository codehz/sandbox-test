#version 450

layout(location = 0) uniform sampler2D color_sample;
layout(location = 1) uniform sampler2D aux_sample;
layout(binding = 0, std140) uniform block { float near, far; };
layout(location = 0) out vec3 color;

const float Directions = 8.0;
const float Quality = 3.0;
const float Pi = 3.14159265359;
const float Tau = 6.28318530718;
const float Size = 0.005;
const vec2 iResolution = textureSize(color_sample, 0);
const vec2 uv = gl_FragCoord.xy / iResolution.xy;
const vec2 Radius = vec2(Size * iResolution.y / iResolution.x, Size);

float rand(vec2 co) {
  return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

vec3 colorSample(in vec2 pos, in float r) {
  vec3 curcolor = texture2D(color_sample, pos).rgb;
  vec2 zx = texture2D(aux_sample, pos).rg;
  float z = zx.r;
  float z_filter = float(z > 0);
  float rz = z_filter * (1.0 - smoothstep(near, far, z));
  float r2 = pow(r, 2.0);
  float rz2 = pow(rz, 2.0);
  return mix(curcolor * r2 * 1.5, curcolor * 1.0 / Quality * rz2 * r,
             smoothstep(rz2, rz2 + 0.1, r)) +
         (curcolor * z_filter * zx.g * 2);
}

void main() {
  color = colorSample(uv, 0.0);

  for (float d = 0.0; d < Tau; d += Tau / Directions) {
    for (float i = 1.0 / Quality; i <= 1.0; i += 1.0 / Quality) {
      vec2 pos = uv + vec2(cos(d), sin(d)) * Radius * i;
      color += colorSample(pos, i);
    }
  }

  color /= Quality * Directions * 0.2;
}