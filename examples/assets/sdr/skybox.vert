#version 450 core
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 inPosition;

layout(location = 1) uniform mat4 view;
layout(location = 2) uniform mat4 projection;

out gl_PerVertex {
    vec4 gl_Position;
};

layout(location = 0) out VsOut {
    vec3 texcoord;
} vsOut;

void main()
{
    // TODO: explain why xyww is used
    mat4 v = view;

    v[0][3] = 0.0;
    v[1][3] = 0.0;
    v[2][3] = 0.0;
    v[3][3] = 1.0;

    // Trick the depth buffer on thinking that the positions are infinitelly far away
    // by set z = w = 1 = max depth value
    gl_Position = (projection * mat4(mat3(v)) * vec4(inPosition, 1.0)).xyww;

    // The cube drawn is centered at the origin so
    // each position is a direction from the origin
    vsOut.texcoord = inPosition;
}
