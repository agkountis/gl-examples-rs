#version 450 core
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 t_InLightDirection;
layout(location = 1) in vec3 t_InViewDirection;
layout(location = 2) in vec2 inTexcoord;

layout(location = 0, binding = 0) uniform sampler2D diffuse;
layout(location = 1, binding = 1) uniform sampler2D specular;
layout(location = 2, binding = 2) uniform sampler2D normal;
layout(location = 3, binding = 3) uniform sampler2D ao;

layout(location = 0) out vec4 outColor;

void main()
{
    vec3 n = normalize(texture(normal, inTexcoord).rgb * 2.0 - 1.0);
    vec3 v = normalize(t_InViewDirection);
    vec3 l = normalize(t_InLightDirection);

    vec3 h = normalize(l + v);

    float diffLight = max(dot(n, l), 0.0);

    float specLight = pow(max(dot(n, h), 0.0), 62.0);

    vec3 diffTexel = texture(diffuse, inTexcoord).rgb;
    vec3 specTexel = texture(specular, inTexcoord).rgb;
    vec3 aoTexel = texture(ao, inTexcoord).rgb;

    outColor = vec4(diffTexel * diffLight + specTexel * specLight + vec3(0.02) * aoTexel, 1.0);
}