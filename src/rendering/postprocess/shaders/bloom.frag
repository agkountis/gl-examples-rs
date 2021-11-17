#version 450 core
#extension GL_ARB_separate_shader_objects : enable

#include "src/rendering/postprocess/shaders/include/engine.glsl"
#include "src/rendering/postprocess/shaders/include/dual_filtering_blur_sampling.glsl"

SAMPLER_2D(0, image);
SAMPLER_2D(1, mainImage);
SAMPLER_2D(2, lensDirt);

UNIFORM_BLOCK_BEGIN(7, BloomParams)
    float spread;
    vec4 filterCurve;
    float intensity;
    int useLensDirt;
    float lensDirtIntensity;
    vec3 tint;
UNIFORM_BLOCK_END

INPUT_BLOCK_BEGIN(0, VsOut)
    vec2 texcoord;
INPUT_BLOCK_END_NAMED(fsIn)

OUTPUT(0, outColor);

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
    vec2 halfpixel = (1.0 / textureSize(image, 0)) * 0.5;
    halfpixel *= spread;

#if defined(BLOOM_PASS_DOWNSAMPLE_PREFILTER)
    vec4 pixelColor = Downsample(image, fsIn.texcoord, halfpixel);
    outColor = vec4(Prefilter(pixelColor.rgb), pixelColor.a);
#elif defined(BLOOM_PASS_DOWNSAMPLE)
    outColor = Downsample(image, fsIn.texcoord, halfpixel);
#elif defined(BLOOM_PASS_UPSAMPLE)
    outColor = Upsample(image, fsIn.texcoord, halfpixel);
#elif defined(BLOOM_PASS_UPSAMPLE_APPLY)
    vec3 bloom = intensity * Upsample(image, fsIn.texcoord, halfpixel).rgb;

    if (useLensDirt == TRUE)
    {
        vec4 lensDirtTexel = texture(lensDirt, fsIn.texcoord);
        vec3 lensDirt = bloom * lensDirtTexel.rgb * lensDirtIntensity;
        bloom += lensDirt;
    }

    bloom *= tint;

    vec4 mainImageColor = texture(mainImage, fsIn.texcoord);

    outColor = min(vec4(mainImageColor.rgb + bloom, mainImageColor.a), vec4(FP16_MAX));
#endif
}

