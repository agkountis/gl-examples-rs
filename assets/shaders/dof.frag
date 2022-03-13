#version 450 core
#extension GL_ARB_separate_shader_objects : enable

#include "assets/shaders/library/engine.glsl"

SAMPLER_2D(0, mainImage);
SAMPLER_2D(1, depthTex);
SAMPLER_2D(2, cocTex);
SAMPLER_2D(3, dofTex);

INPUT_BLOCK_BEGIN(0, VsOut)
    vec2 texcoord;
INPUT_BLOCK_END_NAMED(fsIn)

#ifdef DOF_PASS_COC
    OUTPUT(0, float, outColor);
#else
    OUTPUT(0, vec4, outColor);
#endif

float linearize_depth(float depth)
{
    float zNear = LIB_CAMERA_NEAR_PLANE;
    float zFar = LIB_CAMERA_FAR_PLANE;
    float depthNdc = 2.0 * depth - 1.0;
    return (2.0 * zNear * zFar / (zFar + zNear - depthNdc * (zFar - zNear)));
}

#if defined(DOF_PASS_COC)
    void CoCPass()
    {
        float depth = 0.0;
        depth = linearize_depth(texture(depthTex, fsIn.texcoord).r);// LIB_LINEARIZE_DEPTH(texture(depthTex, fsIn.texcoord).r);

        float coc = clamp((depth - LIB_CAMERA_FOCUS_DISTANCE) / LIB_CAMERA_FOCUS_RANGE, -1.0, 1.0) * LIB_CAMERA_BOKEH_RADIUS;

        outColor = coc;
    }
#elif defined(DOF_PASS_BOKEH)
    #define BOKEH_KERNEL_MEDIUM
    #if defined(BOKEH_KERNEL_SMALL)
        // From https://github.com/Unity-Technologies/PostProcessing/
        // blob/v2/PostProcessing/Shaders/Builtins/DiskKernels.hlsl
        #define KERNEL_SAMPLE_COUNT 16
        const vec2 kernel[KERNEL_SAMPLE_COUNT] = {
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
        #define KERNEL_SAMPLE_COUNT 22
        const vec2 kernel[KERNEL_SAMPLE_COUNT] = {
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

    float Weigh(float coc, float radius)
    {
        return clamp((coc - radius + 2.0) / 2.0, 0.0, 1.0);
    }

    void BokehPass()
    {
        vec3 bgColor = vec3(0.0);
        vec3 fgColor = vec3(0.0);
        float bgWeight = 0.0;
        float fgWeight = 0.0;

        vec2 texelSize =  1.0 / textureSize(mainImage, 0);

        for (int k = 0; k < KERNEL_SAMPLE_COUNT; k++) {
            vec2 offset = kernel[k] * LIB_CAMERA_BOKEH_RADIUS;
            float radius = length(offset);
            offset *= texelSize;
            vec4 s = texture(mainImage, fsIn.texcoord + offset);

            float bgw = Weigh(max(0.0, s.a), radius);
            bgColor += s.rgb * bgw;
            bgWeight += bgw;

            float fgw = Weigh(-s.a, radius);
            fgColor += s.rgb * fgw;
            fgWeight += fgw;
        }

        bgColor *= 1.0 / (bgWeight + float((bgWeight == 0)));
        fgColor *= 1.0 / (fgWeight + float((fgWeight == 0)));

        float bgfg = min(1.0, fgWeight * PI / KERNEL_SAMPLE_COUNT);

        vec3 color = mix(bgColor, fgColor, bgfg);
        outColor = vec4(color, bgfg);
    }
#elif defined(DOF_PASS_DOWNSAMPLE)
    float Weigh(vec3 c)
    {
        return 1.0 / (1.0 + max(max(c.r, c.g), c.b));
    }

    void DownsamplePass()
    {
        vec4 coc_samples = textureGather(cocTex, fsIn.texcoord, 0);

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
#define TONE_DOWN_BOKEH
#ifdef TONE_DOWN_BOKEH
        vec2 texelSize = 1.0 / textureSize(mainImage, 0);
        vec4 offset = texelSize.xyxy * vec2(-0.5, 0.5).xxyy;

        vec3 sample0 = texture(mainImage, fsIn.texcoord + offset.xy).rgb;
        vec3 sample1 = texture(mainImage, fsIn.texcoord + offset.zy).rgb;
        vec3 sample2 = texture(mainImage, fsIn.texcoord + offset.xw).rgb;
        vec3 sample3 = texture(mainImage, fsIn.texcoord + offset.zw).rgb;

        float w0 = Weigh(sample0);
        float w1 = Weigh(sample1);
        float w2 = Weigh(sample2);
        float w3 = Weigh(sample3);

        vec3 color = sample0 * w0 + sample1 * w1 + sample2 * w2 + sample3 * w3;
        color /= max(w0 + w1 + w2 + w3, EPSILON);

        // Premultiply CoC again
        color *= smoothstep(0, texelSize.y * 2.0, abs(coc));

        outColor = vec4(color, coc);
#else
        outColor = vec4(texture(mainImage, fsIn.texcoord).rgb, coc);
#endif
    }
#else
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

    void DofCombinePass()
    {
        vec4 source = texture(mainImage, fsIn.texcoord);
        float coc = texture(cocTex, fsIn.texcoord).r;
        vec4 dof = texture(dofTex, fsIn.texcoord);

        float dofStrength = smoothstep(0.1, 1.0, abs(coc));
        vec3 color = mix(source.rgb, dof.rgb, dofStrength + dof.a - dofStrength * dof.a);
        outColor = vec4(color, source.a);
    }
#endif // DOF_PASS_BOKEH

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
#elif defined(DOF_PASS_COMBINE)
    DofCombinePass();
#endif
}
