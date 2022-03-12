#ifndef CAMERA_GLSL_
#define CAMERA_GLSL_

#include "assets/shaders/library/core_defines.glsl"

UNIFORM_BLOCK_BEGIN(0, CameraData)
    mat4 view;
    mat4 projection;
    mat4 view_projection;
    vec4 cameraPosition;
    vec4 proj_params; // x: near, y:far, z: linearize depth denom, w: linearize depth num.
    vec3 dof_params;
UNIFORM_BLOCK_END

#define LIB_CAMERA_FOCUS_DISTANCE dof_params.x
#define LIB_CAMERA_FOCUS_RANGE dof_params.y
#define LIB_CAMERA_BOKEH_RADIUS dof_params.z

#define LIB_VIEW_PROJECTION_MATRIX view_projection
#define LIB_CAMERA_POSITION cameraPosition
#define LIB_NEAR_PLANE proj_params.x
#define LIB_FAR_PLANE proj_params.y

// https://mynameismjp.wordpress.com/2010/09/05/position-from-depth-3/
#define LIB_LINEARIZE_DEPTH(depth) proj_params.w / (depth - proj_params.z)

#endif // CAMERA_GLSP_
