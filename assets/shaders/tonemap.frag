#version 450 core
#extension GL_ARB_separate_shader_objects : enable

#include "assets/shaders/library/tonemapping.glsl"

layout(binding = 0) uniform sampler2D image;

layout(location = 0) in VsOut {
    vec2 texcoord;
} fsIn;

layout(location = 0) out vec4 outColor;

void main()
{
    vec3 color = texture(image, fsIn.texcoord).rgb * exposure;

    outColor = vec4(TONE_MAP(color), 1.0);
}
