#version 450 core
#extension GL_ARB_separate_shader_objects : enable

#define FP16_MAX 65536.0
#define TRUE 1

layout(binding = 0) uniform sampler2D image;
layout(binding = 1) uniform sampler2D mainImage;
layout(binding = 2) uniform sampler2D lensDirt;

layout(std140, binding = 7) uniform BloomParams
{
    float spread;
    vec4 _filter;
    float intensity;
    int useLensDirt;
    float lensDirtIntensity;
    vec3 tint;
};

layout(location = 0) in VsOut {
    vec2 texcoord;
} fsIn;

layout(location = 0) out vec4 outColor;

vec4 Upsample(vec2 uv, vec2 halfpixel)
{
    vec4 sum = texture(image, uv + vec2(-halfpixel.x * 2.0, 0.0));
    sum += texture(image, uv + vec2(-halfpixel.x, halfpixel.y)) * 2.0;
    sum += texture(image, uv + vec2(0.0, halfpixel.y * 2.0));
    sum += texture(image, uv + vec2(halfpixel.x, halfpixel.y)) * 2.0;
    sum += texture(image, uv + vec2(halfpixel.x * 2.0, 0.0));
    sum += texture(image, uv + vec2(halfpixel.x, -halfpixel.y)) * 2.0;
    sum += texture(image, uv + vec2(0.0, -halfpixel.y * 2.0));
    sum += texture(image, uv + vec2(-halfpixel.x, -halfpixel.y)) * 2.0;
    return sum / 12.0;
}

void main()
{
    vec4 mainImageColor = texture(mainImage, fsIn.texcoord);

//    vec2 halfpixel = texelSize.xy * texelSize.z;
    vec2 halfpixel = (1.0 / textureSize(image, 0)) * 0.5;
    halfpixel *= spread;

    vec3 bloom = intensity * Upsample(fsIn.texcoord, halfpixel).rgb;

    if (useLensDirt == TRUE)
    {
        vec4 lensDirtTexel = texture(lensDirt, fsIn.texcoord);
        vec3 lensDirt = bloom * lensDirtTexel.rgb * lensDirtIntensity;
        bloom += lensDirt;
    }

    bloom *= tint;

    outColor = min(vec4(mainImageColor.rgb + bloom, mainImageColor.a), vec4(FP16_MAX));
}