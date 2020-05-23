#version 450 core
#extension GL_ARB_separate_shader_objects : enable

#define ACES_FITTED

layout(location = 0, binding = 0) uniform sampler2D image;
layout(location = 1, binding = 1) uniform sampler2D bloomImage;
layout(location = 2) uniform float exposure;

in VsOut {
    vec2 texcoord;
} fsIn;

layout(location = 0) out vec4 outColor;

// TONEMAPPING FUNCTIONS ------------------------------------------------
const mat3 ACESInputMat =
{
    {0.59719, 0.35458, 0.04823},
    {0.07600, 0.90834, 0.01566},
    {0.02840, 0.13383, 0.83777}
};

// ODT_SAT => XYZ => D60_2_D65 => sRGB
const mat3 ACESOutputMat =
{
    { 1.60475, -0.53108, -0.07367},
    {-0.10208,  1.10813, -0.00605},
    {-0.00327, -0.07276,  1.07602}
};

vec3 RRTAndODTFit(vec3 v)
{
    vec3 a = v * (v + 0.0245786f) - 0.000090537f;
    vec3 b = v * (0.983729f * v + 0.4329510f) + 0.238081f;
    return a / b;
}

// Complex fit. Better for realistic rendering
vec3 ACESFitted(vec3 color)
{
    color = ACESInputMat * color;

    // Apply RRT and ODT
    color = RRTAndODTFit(color);

    color = ACESOutputMat * color;

    // Clamp to [0, 1]
    color = clamp(color, 0.0, 1.0);

    return color;
}

//simple luminance fit. Oversaturates brights
vec3 ACESFilm(vec3 x)
{
    float a = 2.51f;
    float b = 0.03f;
    float c = 2.43f;
    float d = 0.59f;
    float e = 0.14f;
    return clamp((x*(a*x+b))/(x*(c*x+d)+e), 0.0, 1.0);
}

void main()
{
    vec3 color = (texture(image, fsIn.texcoord).rgb + texture(bloomImage, fsIn.texcoord).rgb/*TODO: make this a uniform*/) * exposure;
#ifdef ACES_FITTED
    outColor = vec4(ACESFitted(color), 1.0);
//    outColor = vec4(texture(bloomImage, fsIn.texcoord).rgb, 1.0);
#else
    outColor = vec4(ACESFilm(color), 1.0);
#endif
}
