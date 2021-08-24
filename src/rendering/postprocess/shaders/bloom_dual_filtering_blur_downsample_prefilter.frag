#version 450 core
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0, binding = 0) uniform sampler2D image;

layout(std140, binding = 7) uniform BloomParams
{
    vec4 _filter;
};

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

vec3 Prefilter (vec3 c) {
    float brightness = max(c.r, max(c.g, c.b));
    float soft = brightness - _filter.y;
    soft = clamp(soft, 0, _filter.z);
    soft = soft * soft * _filter.w;
    float contribution = max(soft, brightness - _filter.x);
    contribution /= max(brightness, 0.00001);
    return c * contribution;
}

void main()
{
    vec2 halfpixel = (1.0 / textureSize(image, 0)) * 0.5;
    vec4 pixelColor = Downsample(fsIn.texcoord, halfpixel);
    outColor = vec4(Prefilter(pixelColor.rgb), pixelColor.a);
}
