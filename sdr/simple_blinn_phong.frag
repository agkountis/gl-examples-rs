#version 450 core
#extension GL_ARB_separate_shader_objects : enable

#define MAX_LIGHTS 1

struct DirectionalLight {
    vec3 directionl;
    vec3 diffuse;
    vec3 specular;
    vec3 ambient;
};

layout(location = 0) in vec2 inTexcoord;

layout(std140, binding = 0) uniform DirectionalLights {
    DirectionalLight[MAX_LIGHTS] directionalLights;
} lightsUbo;

layout(location = 0, binding = 0) uniform sampler2D albedo;

layout(location = 0) out vec4 outColor;

void main()
{
    vec3 albedo_color = texture(albedo, inTexcoord).rgb;

    outColor = vec4(albedo_color, 1.0);
}
