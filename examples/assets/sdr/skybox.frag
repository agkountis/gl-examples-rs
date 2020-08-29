#version 450 core
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in VsOut {
    vec3 texcoord;
} fsIn;

layout(binding = 0) uniform samplerCube skybox;

layout(location = 0) out vec4 outColor;

void main()
{
    outColor = texture(skybox, fsIn.texcoord);
}
