#version 450 core
#extension GL_ARB_separate_shader_objects : enable

#include "assets/shaders/library/engine.glsl"

SAMPLER_2D(0, mainImage);
SAMPLER_2D(1, depthTex);
SAMPLER_2D(2, cocTex);

INPUT_BLOCK_BEGIN(0, VsOut)
    vec2 texcoord;
INPUT_BLOCK_END_NAMED(fsIn)

#ifdef DOF_PASS_COC
    OUTPUT(0, float, outColor);
#else
    OUTPUT(0, vec4, outColor);
#endif


#if defined(DOF_PASS_COC)
    void CoCPass()
    {
        float depth = 0.0;
        depth = LIB_LINEARIZE_DEPTH(texture(depthTex, fsIn.texcoord).r);

        float coc = clamp((depth - LIB_CAMERA_FOCUS_DISTANCE) / LIB_CAMERA_FOCUS_RANGE, -1.0, 1.0) * LIB_CAMERA_BOKEH_RADIUS;

        outColor = coc;
    }
#elif defined(DOF_PASS_BOKEH)
    #define BOKEH_KERNEL_MEDIUM
    #if defined(BOKEH_KERNEL_SMALL)
        // From https://github.com/Unity-Technologies/PostProcessing/
        // blob/v2/PostProcessing/Shaders/Builtins/DiskKernels.hlsl
        const int kernelSampleCount = 16;
        const vec2 kernel[kernelSampleCount] = {
            vec2(0, 0),
            vec2(0.54545456, 0),
            vec2(0.16855472, 0.5187581),
            vec2(-0.44128203, 0.3206101),
            vec2(-0.44128197, -0.3206102),
            vec2(0.1685548, -0.5187581),
            vec2(1, 0),
            vec2(0.809017, 0.58778524),
            vec2(0.30901697, 0.95105654),
            vec2(-0.30901703, 0.9510565),
            vec2(-0.80901706, 0.5877852),
            vec2(-1, 0),
            vec2(-0.80901694, -0.58778536),
            vec2(-0.30901664, -0.9510566),
            vec2(0.30901712, -0.9510565),
            vec2(0.80901694, -0.5877853),
        };
    #elif defined(BOKEH_KERNEL_MEDIUM)
        const int kernelSampleCount = 22;
        const vec2 kernel[kernelSampleCount] = {
            vec2(0, 0),
            vec2(0.53333336, 0),
            vec2(0.3325279, 0.4169768),
            vec2(-0.11867785, 0.5199616),
            vec2(-0.48051673, 0.2314047),
            vec2(-0.48051673, -0.23140468),
            vec2(-0.11867763, -0.51996166),
            vec2(0.33252785, -0.4169769),
            vec2(1, 0),
            vec2(0.90096885, 0.43388376),
            vec2(0.6234898, 0.7818315),
            vec2(0.22252098, 0.9749279),
            vec2(-0.22252095, 0.9749279),
            vec2(-0.62349, 0.7818314),
            vec2(-0.90096885, 0.43388382),
            vec2(-1, 0),
            vec2(-0.90096885, -0.43388376),
            vec2(-0.6234896, -0.7818316),
            vec2(-0.22252055, -0.974928),
            vec2(0.2225215, -0.9749278),
            vec2(0.6234897, -0.7818316),
            vec2(0.90096885, -0.43388376),
        };
    #endif //BOKEH_KERNEL_SMALL

    void BokehPass()
    {
        vec3 color = vec3(0.0);
        vec2 texelSize = 1.0 / textureSize(mainImage, 0);
        float weight = 0.0;

        for (int k = 0; k < kernelSampleCount; k++) {
            vec2 offset = kernel[k] * LIB_CAMERA_BOKEH_RADIUS;
            float radius = length(offset);
            offset *= texelSize;
            vec4 s = texture(mainImage, fsIn.texcoord + offset);

            if (abs(s.a) >= radius) {
                color += s.rgb;
                weight += 1.0;
            }
        }

        color *= 1.0 / weight;
        outColor = vec4(color, 1.0);
    }
#else
    void DownsamplePass()
    {
        vec4 coc_samples = textureGather(cocTex, fsIn.texcoord);

        float coc_min = min(
            min(
                min(coc_samples.x, coc_samples.y),
                coc_samples.z
            ),
            coc_samples.w
        );
        float coc_max = max(
            max(
                max(coc_samples.x, coc_samples.y),
                coc_samples.z
            ),
            coc_samples.w
        );

        float coc = coc_max >= -coc_min ? coc_max : coc_min;

        outColor = vec4(texture(mainImage, fsIn.texcoord).rgb, coc);
    }

    void BokehBlurPass()
    {
        vec2 texelSize = 1.0 / textureSize(mainImage, 0);
        vec4 offset = texelSize.xyxy * vec2(-0.5, 0.5).xxyy;
        vec4 color =
        texture(mainImage, fsIn.texcoord + offset.xy) +
        texture(mainImage, fsIn.texcoord + offset.zy) +
        texture(mainImage, fsIn.texcoord + offset.xw) +
        texture(mainImage, fsIn.texcoord + offset.zw);

        outColor = color * 0.25;
    }
#endif // DOF_PASS_BOKEH


#ifdef DOF_PASS_COC

#else

#endif

void main()
{
#if defined(DOF_PASS_COC)
    CoCPass();
#elif defined(DOF_PASS_DOWNSAMPLE)
    DownsamplePass();
#elif defined(DOF_PASS_BOKEH)
    BokehPass();
#elif defined(DOF_PASS_BOKEH_BLUR)
    BokehBlurPass();
#endif
}
