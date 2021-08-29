#version 450 core
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 inPosition;

layout(std140, binding = 0) uniform CameraMatrices
{
    mat4 view;
    mat4 projection;
};

out gl_PerVertex {
    vec4 gl_Position;
};

layout(location = 0) out VsOut {
    vec3 texcoord;
} vsOut;

void main()
{
    // Trick the depth buffer on thinking that the positions are infinitelly far away
    // by set z = w = 1 = max depth value
    mat4 v = mat4(mat3(view));
    gl_Position = (projection * v * vec4(inPosition, 1.0)).xyww;

    // The cube drawn is centered at the origin so
    // each position is a direction from the origin
    vsOut.texcoord = inPosition;
}
