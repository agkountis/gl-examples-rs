#version 450 core
#extension GL_ARB_separate_shader_objects : enable
//
//layout(location = 0) in vec3 t_InLightDirection;
//layout(location = 1) in vec3 t_InViewDirection;
//layout(location = 2) in vec2 inTexcoord;

in VsOut {
    vec3 tLightDirection;
    vec3 tViewDirection;
    vec2 texcoord;
} fsIn;

layout(location = 0, binding = 0) uniform sampler2D diffuse;
layout(location = 1, binding = 1) uniform sampler2D specular;
layout(location = 2, binding = 2) uniform sampler2D normal;
layout(location = 3, binding = 3) uniform sampler2D ao;

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
vec3 ACESFilm( vec3 x )
{
    float a = 2.51f;
    float b = 0.03f;
    float c = 2.43f;
    float d = 0.59f;
    float e = 0.14f;
    return clamp((x*(a*x+b))/(x*(c*x+d)+e), 0.0, 1.0);
}
// END TONEMAPPING FUNCTIONS --------------------------------------

// PBS FUNCTIONS --------------------------------------------------
// END PBS FUNCTIONS ----------------------------------------------

void main()
{
    vec3 n = normalize(texture(normal, fsIn.texcoord).rgb * 2.0 - 1.0);
    vec3 v = normalize(fsIn.tViewDirection);
    vec3 l = normalize(fsIn.tLightDirection);

    vec3 h = normalize(l + v);

    float diffLight = max(dot(n, l), 0.0);

    float specLight = pow(max(dot(n, h), 0.0), 30.0);

    vec3 li = vec3(6.0);
    vec3 diffColor = texture(diffuse, fsIn.texcoord).rgb * diffLight * li;
    vec3 specColor = texture(specular, fsIn.texcoord).rgb * specLight * li;
    vec3 aoTexel = texture(ao, fsIn.texcoord).rgb;

    // Tone map with ACES filter.
    outColor = vec4(ACESFitted(diffColor + specColor + vec3(0.02) * aoTexel), 1.0);
}