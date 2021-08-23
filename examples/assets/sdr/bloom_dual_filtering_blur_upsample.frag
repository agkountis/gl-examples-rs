#version 450 core
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0, binding = 0) uniform sampler2D image;

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
    vec2 halfpixel = (1.0 / textureSize(image, 0)) * 0.5;
    outColor = Upsample(fsIn.texcoord, halfpixel);
}
