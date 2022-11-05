#ifndef SAMPLING_GLSL_
#define SAMPLING_GLSL_

#include "assets/shaders/library/utilities.glsl"

vec3 SampleNormalMap(in sampler2D normalMap, in vec2 texcoords, in float strength)
{
    vec3 norm = texture(normalMap, texcoords).rgb * 2.0 - 1.0;
    norm.xy *= strength;
    return normalize(norm);
}

vec3 DualFilteringDownsample(sampler2D image, vec2 uv, vec2 halfpixel)
{
    vec3 sum = texture(image, uv).rgb * 4.0;
    sum += texture(image, uv - halfpixel.xy).rgb;
    sum += texture(image, uv + halfpixel.xy).rgb;
    sum += texture(image, uv + vec2(halfpixel.x, -halfpixel.y)).rgb;
    sum += texture(image, uv - vec2(halfpixel.x, -halfpixel.y)).rgb;
    return sum / 8.0;
}

vec3 DualFilteringDownsampleWithKarisAverage(sampler2D image, vec2 uv, vec2 halfpixel)
{
    vec3 s1 = texture(image, uv).rgb * 4.0;
    vec3 s2 = texture(image, uv - halfpixel.xy).rgb;
    vec3 s3 = texture(image, uv + halfpixel.xy).rgb;
    vec3 s4 = texture(image, uv + vec2(halfpixel.x, -halfpixel.y)).rgb;
    vec3 s5 = texture(image, uv - vec2(halfpixel.x, -halfpixel.y)).rgb;

    s1 *= KarisAverage(s1, 1.0);
    s2 *= KarisAverage(s2, 1.0);
    s3 *= KarisAverage(s3, 1.0);
    s4 *= KarisAverage(s4, 1.0);
    s5 *= KarisAverage(s5, 1.0);

    return (s1 + s2 + s3 + s4 + s5) / 8.0;
}

vec3 DualFilteringUpsample(sampler2D image, vec2 uv, vec2 halfpixel)
{
    vec3 sum = texture(image, uv + vec2(-halfpixel.x * 2.0, 0.0)).rgb;
    sum += texture(image, uv + vec2(-halfpixel.x, halfpixel.y)).rgb * 2.0;
    sum += texture(image, uv + vec2(0.0, halfpixel.y * 2.0)).rgb;
    sum += texture(image, uv + vec2(halfpixel.x, halfpixel.y)).rgb * 2.0;
    sum += texture(image, uv + vec2(halfpixel.x * 2.0, 0.0)).rgb;
    sum += texture(image, uv + vec2(halfpixel.x, -halfpixel.y)).rgb * 2.0;
    sum += texture(image, uv + vec2(0.0, -halfpixel.y * 2.0)).rgb;
    sum += texture(image, uv + vec2(-halfpixel.x, -halfpixel.y)).rgb * 2.0;
    return sum / 12.0;
}

vec3 CoD_AW_FilteringDownsample(sampler2D image, vec2 uv, vec2 texelSize)
{
    float x = texelSize.x;
    float y = texelSize.y;

    // Take 13 samples around current texel:
    // a - b - c
    // - j - k -
    // d - e - f
    // - l - m -
    // g - h - i
    // === ('e' is the current texel) ===
    vec3 a = texture(image, vec2(uv.x - 2.0 * x, uv.y + 2.0 * y)).rgb;
    vec3 b = texture(image, vec2(uv.x,           uv.y + 2.0 * y)).rgb;
    vec3 c = texture(image, vec2(uv.x + 2.0 * x, uv.y + 2.0 * y)).rgb;

    vec3 d = texture(image, vec2(uv.x - 2.0 * x, uv.y)).rgb;
    vec3 e = texture(image, vec2(uv.x,           uv.y)).rgb;
    vec3 f = texture(image, vec2(uv.x + 2.0 * x, uv.y)).rgb;

    vec3 g = texture(image, vec2(uv.x - 2.0 * x, uv.y - 2.0 * y)).rgb;
    vec3 h = texture(image, vec2(uv.x,           uv.y - 2.0 * y)).rgb;
    vec3 i = texture(image, vec2(uv.x + 2.0 * x, uv.y - 2.0 * y)).rgb;

    vec3 j = texture(image, vec2(uv.x - x, uv.y + y)).rgb;
    vec3 k = texture(image, vec2(uv.x + x, uv.y + y)).rgb;
    vec3 l = texture(image, vec2(uv.x - x, uv.y - y)).rgb;
    vec3 m = texture(image, vec2(uv.x + x, uv.y - y)).rgb;

    // Apply weighted distribution:
    // 0.5 + 0.125 + 0.125 + 0.125 + 0.125 = 1
    // a,b,d,e * 0.125
    // b,c,e,f * 0.125
    // d,e,g,h * 0.125
    // e,f,h,i * 0.125
    // j,k,l,m * 0.5
    // This shows 5 square areas that are being sampled. But some of them overlap,
    // so to have an energy preserving downsample we need to make some adjustments.
    // The weights are the distributed, so that the sum of j,k,l,m (e.g.)
    // contribute 0.5 to the final color output. The code below is written
    // to effectively yield this sum. We get:
    // 0.125*5 + 0.03125*4 + 0.0625*4 = 1

    vec3 sum;
    sum = e * 0.125;
    sum += (a + c + g + i) * 0.03125;
    sum += (b + d + f + h) * 0.0625;
    sum += (j + k + l + m) * 0.125;

    return sum;
}

vec3 CoD_AW_FilteringDownsampleWithKarisAverage(sampler2D image, vec2 uv, vec2 texelSize)
{
    float x = texelSize.x;
    float y = texelSize.y;

    // Take 13 samples around current texel:
    // a - b - c
    // - j - k -
    // d - e - f
    // - l - m -
    // g - h - i
    // === ('e' is the current texel) ===
    vec3 a = texture(image, vec2(uv.x - 2.0 * x, uv.y + 2.0 * y)).rgb;
    vec3 b = texture(image, vec2(uv.x,           uv.y + 2.0 * y)).rgb;
    vec3 c = texture(image, vec2(uv.x + 2.0 * x, uv.y + 2.0 * y)).rgb;

    vec3 d = texture(image, vec2(uv.x - 2.0 * x, uv.y)).rgb;
    vec3 e = texture(image, vec2(uv.x,           uv.y)).rgb;
    vec3 f = texture(image, vec2(uv.x + 2.0 * x, uv.y)).rgb;

    vec3 g = texture(image, vec2(uv.x - 2.0 * x, uv.y - 2.0 * y)).rgb;
    vec3 h = texture(image, vec2(uv.x,           uv.y - 2.0 * y)).rgb;
    vec3 i = texture(image, vec2(uv.x + 2.0 * x, uv.y - 2.0 * y)).rgb;

    vec3 j = texture(image, vec2(uv.x - x, uv.y + y)).rgb;
    vec3 k = texture(image, vec2(uv.x + x, uv.y + y)).rgb;
    vec3 l = texture(image, vec2(uv.x - x, uv.y - y)).rgb;
    vec3 m = texture(image, vec2(uv.x + x, uv.y - y)).rgb;

    vec3 groups[5];
    groups[0] = (a + b + d + e) * (0.125f/4.0f);
    groups[1] = (b + c + e + f) * (0.125f/4.0f);
    groups[2] = (d + e + g + h) * (0.125f/4.0f);
    groups[3] = (e + f + h + i) * (0.125f/4.0f);
    groups[4] = (j + k + l + m) * (0.5f/4.0f);
    groups[0] *= KarisAverage(groups[0], 4);
    groups[1] *= KarisAverage(groups[1], 4);
    groups[2] *= KarisAverage(groups[2], 4);
    groups[3] *= KarisAverage(groups[3], 4);
    groups[4] *= KarisAverage(groups[4], 4);

    vec3 sum;
    sum = groups[0] + groups[1] + groups[2] + groups[3] + groups[4];
    sum = max(sum, 0.0001);

    return sum;
}

vec3 CoD_AW_FilteringUpsample(sampler2D image, vec2 uv, vec2 pixelSize, float filterRadius)
{
    // The filter kernel is applied with a radius, specified in texture
    // coordinates, so that the radius will vary across mip resolutions.
    float x = pixelSize.x * filterRadius;
    float y = pixelSize.y * filterRadius;

    // Take 9 samples around current texel:
    // a - b - c
    // d - e - f
    // g - h - i
    // === ('e' is the current texel) ===
    vec3 a = texture(image, vec2(uv.x - x, uv.y + y)).rgb;
    vec3 b = texture(image, vec2(uv.x,     uv.y + y)).rgb;
    vec3 c = texture(image, vec2(uv.x + x, uv.y + y)).rgb;

    vec3 d = texture(image, vec2(uv.x - x, uv.y)).rgb;
    vec3 e = texture(image, vec2(uv.x,     uv.y)).rgb;
    vec3 f = texture(image, vec2(uv.x + x, uv.y)).rgb;

    vec3 g = texture(image, vec2(uv.x - x, uv.y - y)).rgb;
    vec3 h = texture(image, vec2(uv.x,     uv.y - y)).rgb;
    vec3 i = texture(image, vec2(uv.x + x, uv.y - y)).rgb;

    // Apply weighted distribution, by using a 3x3 tent filter:
    //  1   | 1 2 1 |
    // -- * | 2 4 2 |
    // 16   | 1 2 1 |
    vec3 sum;
    sum = e * 4.0;
    sum += (b + d + f + h) * 2.0;
    sum += a + c + g + i;
    sum *= 1.0 / 16.0;

    return sum;
}

#endif //SAMPLING_GLSL_