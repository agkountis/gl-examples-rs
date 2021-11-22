#ifndef DUAL_FILTERING_BLUR_GLSL_
#define DUAL_FILTERING_BLUR_GLSL_

vec4 Downsample(sampler2D image, vec2 uv, vec2 halfpixel)
{
    vec4 sum = texture(image, uv) * 4.0;
    sum += texture(image, uv - halfpixel.xy);
    sum += texture(image, uv + halfpixel.xy);
    sum += texture(image, uv + vec2(halfpixel.x, -halfpixel.y));
    sum += texture(image, uv - vec2(halfpixel.x, -halfpixel.y));
    return sum / 8.0;
}

vec4 Upsample(sampler2D image, vec2 uv, vec2 halfpixel)
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

#endif //DUAL_FILTERING_BLUR_GLSL_