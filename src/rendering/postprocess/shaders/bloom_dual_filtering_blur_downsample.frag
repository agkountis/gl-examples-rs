#version 450 core
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0, binding = 0) uniform sampler2D image;

layout(location = 0) in VsOut {
    vec2 texcoord;
} fsIn;

layout(location = 0) out vec4 outColor;

vec4 Downsample(vec2 uv, vec2 halfpixel)
{
    vec4 sum = texture(image, uv) * 4.0;
    sum += texture(image, uv - halfpixel.xy);
    sum += texture(image, uv + halfpixel.xy);
    sum += texture(image, uv + vec2(halfpixel.x, -halfpixel.y));
    sum += texture(image, uv - vec2(halfpixel.x, -halfpixel.y));
    return sum / 8.0;
}

void main()
{
    vec2 halfpixel = (1.0 / textureSize(image, 0)) * 0.5;
    outColor = Downsample(fsIn.texcoord, halfpixel);
}
