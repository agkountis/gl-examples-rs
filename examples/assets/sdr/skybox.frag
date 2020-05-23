#version 450 core
#extension GL_ARB_separate_shader_objects : enable

in VsOut {
    vec3 texcoord;
} fsIn;

layout(location = 0, binding = 0) uniform samplerCube skybox;

layout(location = 0) out vec4 outColor;

void main()
{
    outColor = texture(skybox, fsIn.texcoord);
}
