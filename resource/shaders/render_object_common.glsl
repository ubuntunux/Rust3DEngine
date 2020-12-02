layout(binding = 0) uniform SceneConstants
{
    SCENE_CONSTANTS scene_constants;
};
layout(binding = 1) uniform ViewConstants
{
    VIEW_CONSTANTS view_constants;
};
layout(binding = 2) uniform LightConstants
{
    LIGHT_CONSTANTS light_constants;
};
layout(binding = 3) buffer BoneConstants
{
    mat4 bone_matrices[MAX_BONES];
};

layout( push_constant ) uniform PushConstant
{
    mat4 _localMatrix;
    uint _bone_matrix_offset;
    uint _bone_matrix_count;
    uint _reserved0;
    uint _reserved1;
} pushConstant;

struct VERTEX_OUTPUT
{
    mat3 tangent_to_world;
    vec4 color;
    vec2 texCoord;
    vec4 projection_pos;
    vec4 projection_pos_prev;
};