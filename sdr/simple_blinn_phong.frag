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

layout(location = 0) out vec4 outColor;

void main()
{
    outColor = vec4(inTexcoord, 0.0, 1.0);
}
