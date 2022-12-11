#version 450 core
#extension GL_ARB_separate_shader_objects : enable

#include "assets/shaders/library/engine.glsl"
#include "assets/shaders/library/sampling.glsl"

SAMPLER_2D(0, image);

#ifdef BLOOM_PASS_UPSAMPLE_APPLY
    SAMPLER_2D(1, mainImage);
    SAMPLER_2D(2, lensDirt);
#endif

UNIFORM_BLOCK_BEGIN(7, BloomParams)
    float spread;
    vec4 filterCurve;
    float filterRadius;
    float intensity;
    int useLensDirt;
    float lensDirtIntensity;
    vec3 tint;
UNIFORM_BLOCK_END

INPUT_BLOCK_BEGIN(0, VsOut)
    vec2 texcoord;
INPUT_BLOCK_END_NAMED(fsIn)

OUTPUT(0, vec4, outColor);

vec3 Prefilter (vec3 c) {
    float brightness = LumaRec709(c);//max(c.r, max(c.g, c.b));
    float soft = brightness - filterCurve.y;
    soft = clamp(soft, 0, filterCurve.z);
    soft = soft * soft * filterCurve.w;
    float contribution = max(soft, brightness - filterCurve.x);
    contribution /= max(brightness, 0.00001);
    return c * contribution;
}

#ifdef COD_AW_FILTERING
    #ifdef BLOOM_PASS_DOWNSAMPLE_PREFILTER
        #define DOWNSAMPLE_FUNC(image, uv, texelSize) CoD_AW_FilteringDownsampleWithKarisAverage(image, uv, texelSize)
    #else
        #define DOWNSAMPLE_FUNC(image, uv, texelSize) CoD_AW_FilteringDownsample(image, uv, texelSize)
    #endif

    #define UPSAMPLE_FUNC(image, uv, texelSize) CoD_AW_FilteringUpsample(image, uv, texelSize, filterRadius)
#else
    #ifdef BLOOM_PASS_DOWNSAMPLE_PREFILTER
        #define DOWNSAMPLE_FUNC(image, uv, texelSize) DualFilteringDownsampleWithKarisAverage(image, uv, texelSize)
    #else
        #define DOWNSAMPLE_FUNC(image, uv, texelSize) DualFilteringDownsample(image, uv, texelSize)
    #endif

    #define UPSAMPLE_FUNC(image, uv, texelSize) DualFilteringUpsample(image, uv, texelSize)
#endif

void main()
{
#ifdef COD_AW_FILTERING
    float texelSizeScalar = 1.0;
#else
    float texelSizeScalar = 0.5;
#endif

    vec2 texelSize = 1.0 / textureSize(image, 0) * texelSizeScalar;
    texelSize *= spread;

#if defined(BLOOM_PASS_DOWNSAMPLE_PREFILTER)
    vec3 pixelColor = max(DOWNSAMPLE_FUNC(image, fsIn.texcoord, texelSize), 0.0001);
    outColor = vec4(Prefilter(pixelColor), 1.0);
#elif defined(BLOOM_PASS_DOWNSAMPLE)
    outColor = vec4(DOWNSAMPLE_FUNC(image, fsIn.texcoord, texelSize), 1.0);
#elif defined(BLOOM_PASS_UPSAMPLE)
    outColor = vec4(UPSAMPLE_FUNC(image, fsIn.texcoord, texelSize), 1.0);
#elif defined(BLOOM_PASS_UPSAMPLE_APPLY)
    vec3 bloom = intensity * UPSAMPLE_FUNC(image, fsIn.texcoord, texelSize).rgb;

    if (useLensDirt == TRUE)
    {
        vec4 lensDirtTexel = texture(lensDirt, fsIn.texcoord);
        vec3 lensDirt = bloom * lensDirtTexel.rgb * lensDirtIntensity;
        bloom += lensDirt;
    }

    bloom *= tint;

    vec4 mainImageColor = texture(mainImage, fsIn.texcoord);

    vec3 combinedColor = min(mainImageColor.rgb + bloom, vec3(FP16_MAX));
    outColor = vec4(mix(mainImageColor.rgb, combinedColor, intensity), mainImageColor.a);
#endif
}

