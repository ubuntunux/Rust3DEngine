#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive : enable

#include "utility.glsl"
#include "scene_constants.glsl"
#include "render_quad_common.glsl"

// VkImageViewType
#define VK_IMAGE_VIEW_TYPE_1D 0
#define VK_IMAGE_VIEW_TYPE_2D 1
#define VK_IMAGE_VIEW_TYPE_3D 2
#define VK_IMAGE_VIEW_TYPE_CUBE 3
#define VK_IMAGE_VIEW_TYPE_1D_ARRAY 4
#define VK_IMAGE_VIEW_TYPE_2D_ARRAY 5
#define VK_IMAGE_VIEW_TYPE_CUBE_ARRAY 6

layout(binding = 0) uniform ViewConstants
{
    VIEW_CONSTANTS view_constants;
};
layout(binding = 1) uniform sampler2D texture_2d;
layout(binding = 2) uniform sampler2DArray texture_2d_array;
layout(binding = 3) uniform sampler3D texture_3d;
layout(binding = 4) uniform samplerCube texture_cube;

layout( push_constant ) uniform PushConstant_Debug
{
    uint _debug_target;
    uint _reserved0;
    uint _reserved1;
    uint _reserved2;
} pushConstant;


layout(location = 0) in VERTEX_OUTPUT vs_output;

layout(location = 0) out vec4 outColor;

vec4 get_texture_2d_array(sampler2DArray texture_source)
{
    vec3 texture_size = textureSize(texture_source, 0);
    float width = ceil(sqrt(texture_size.z));
    float height = ceil(texture_size.z / width);
    float layer = floor(vs_output.texCoord.x * width) + floor((1.0 - vs_output.texCoord.y) * height) * width;
    if(texture_size.z <= layer)
    {
        return vec4(0.0, 0.0, 0.0, 0.0);
    }
    vec3 texcoord = vec3(fract(vs_output.texCoord.x * width), fract(vs_output.texCoord.y * height), layer);
    return texture(texture_source, texcoord);
}

vec4 get_texture_3d(sampler3D texture_source)
{
    vec3 texture_size = textureSize(texture_source, 0);
    float width = ceil(sqrt(texture_size.z));
    float height = ceil(texture_size.z / width);
    float depth = floor(vs_output.texCoord.x * width) + floor((1.0 - vs_output.texCoord.y) * height) * width;
    if(texture_size.z <= depth)
    {
        return vec4(0.0, 0.0, 0.0, 0.0);
    }
    depth /= texture_size.z;
    vec3 texcoord = vec3(fract(vs_output.texCoord.x * width), fract(vs_output.texCoord.y * height), depth);
    return texture(texture_source, texcoord);
}

void main() {
    uint debug_target = pushConstant._debug_target;

    if(VK_IMAGE_VIEW_TYPE_2D == debug_target)
    {
        vec2 texcoord = vs_output.texCoord.xy;
        outColor = texture(texture_2d, texcoord);
    }
    else if(VK_IMAGE_VIEW_TYPE_2D_ARRAY == debug_target)
    {
        outColor = get_texture_2d_array(texture_2d_array);
    }
    else if(VK_IMAGE_VIEW_TYPE_3D == debug_target)
    {
        outColor = get_texture_3d(texture_3d);
    }
    else if(VK_IMAGE_VIEW_TYPE_CUBE == debug_target)
    {
        vec4 position = vec4(vs_output.texCoord.xy * 2.0 - 1.0, -1.0, 1.0);
        position = view_constants.INV_VIEW_ORIGIN_PROJECTION * position;
        position.xyz /= position.w;
        outColor = texture(texture_cube, normalize(position.xyz));
    }
    else
    {
        vec2 texcoord = vs_output.texCoord.xy;
        outColor = texture(texture_2d, texcoord);
    }

//    if(debug_absolute)
//    {
//        outColor.xyz = abs(outColor.xyz);
//    }

//    outColor.xyz = clamp((outColor.xyz - debug_intensity_min) / (debug_intensity_max - debug_intensity_min), 0.0, 1.0);
    outColor.xyz = pow(clamp(outColor.xyz, 0.0, 1.0), vec3(2.2));
    outColor.w = 1.0;
}