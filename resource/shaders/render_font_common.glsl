// MAX_FONT_INSTANCE_COUNT must match with font.rs
const uint MAX_FONT_INSTANCE_COUNT = 1024;

layout( push_constant ) uniform PushConstant_RenderFont
{
    vec2 _offset;
    vec2 _inv_canvas_size;
    float _font_size;
    float _count_of_side;
    uint reserved0;
    uint reserved1;
} pushConstant;

layout(binding = 0) uniform sampler2D texture_font;
layout(binding = 1) buffer FontInstanceData
{
    vec4 font_instance_infos[MAX_FONT_INSTANCE_COUNT];
};


struct VERTEX_OUTPUT
{
    vec2 texcoord;
    vec2 font_offset;
};