#version 450 core
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 v_InLightDirection;
layout(location = 1) in vec3 v_InViewDirection;
layout(location = 2) in vec2 inTexcoord;
layout(location = 3) in vec3 inNormal;
layout(location = 4) in vec3 inVertexColor;

layout(location = 0, binding = 0) uniform sampler2D diffuse;
layout(location = 1, binding = 1) uniform sampler2D specular;
layout(location = 2, binding = 2) uniform sampler2D normal;

layout(location = 0) out vec4 outColor;

void main()
{
    vec3 n = normalize(texture(normal, inTexcoord).rgb * 2.0 - 1.0);
    vec3 v = normalize(v_InViewDirection);
    vec3 l = normalize(v_InLightDirection);

    vec3 h = normalize(l + v);

    float diffLight = max(dot(n, l), 0.0);

    float specLight = pow(max(dot(n, h), 0.0), 60.0);

    vec4 diffTexel = texture(diffuse, inTexcoord);
    vec4 specTexel = texture(specular, inTexcoord);

    outColor = diffTexel * diffLight + specTexel * specLight;
}