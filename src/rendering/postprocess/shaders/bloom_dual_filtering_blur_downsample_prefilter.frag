#version 450 core
#extension GL_ARB_separate_shader_objects : enable

#include "src/rendering/postprocess/shaders/include/dual_filtering_blur_sampling.glsl"

layout(binding = 0) uniform sampler2D image;

layout(std140, binding = 7) uniform BloomParams
{
    float spread;
    vec4 filterCurve;
};

layout(location = 0) in VsOut {
    vec2 texcoord;
} fsIn;

layout(location = 0) out vec4 outColor;

vec3 Prefilter (vec3 c) {
    float brightness = max(c.r, max(c.g, c.b));
    float soft = brightness - filterCurve.y;
    soft = clamp(soft, 0, filterCurve.z);
    soft = soft * soft * filterCurve.w;
    float contribution = max(soft, brightness - filterCurve.x);
    contribution /= max(brightness, 0.00001);
    return c * contribution;
}

void main()
{
//    vec2 halfpixel = texelSize.xy * texelSize.z;
    vec2 halfpixel = (1.0 / textureSize(image, 0)) * 0.5;
    halfpixel *= spread;

    vec4 pixelColor = Downsample(image, fsIn.texcoord, halfpixel);
    outColor = vec4(Prefilter(pixelColor.rgb), pixelColor.a);
}
