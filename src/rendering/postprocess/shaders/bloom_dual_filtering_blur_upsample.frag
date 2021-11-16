#version 450 core
#extension GL_ARB_separate_shader_objects : enable

#include "src/rendering/postprocess/shaders/include/dual_filtering_blur_sampling.glsl"

layout(binding = 0) uniform sampler2D image;

layout(std140, binding = 7) uniform BloomParams
{
    float spread;
};

layout(location = 0) in VsOut {
    vec2 texcoord;
} fsIn;

layout(location = 0) out vec4 outColor;

void main()
{
//    vec2 halfpixel = texelSize.xy * texelSize.z;
    vec2 halfpixel = (1.0 / textureSize(image, 0)) * 0.5;
    halfpixel *= spread;

    outColor = Upsample(image, fsIn.texcoord, halfpixel);
}
