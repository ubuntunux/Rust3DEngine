#define INSTANCE_ID_LOCATION 3

// must match with ui.rs
const uint MAX_UI_INSTANCE_COUNT = 1024;
const uint UI_RENDER_FLAG_NONE = 0;
const uint UI_RENDER_FLAG_RENDER_TEXT = 1 << 0;

struct VERTEX_OUTPUT
{
    vec4 _color;
    vec4 _border_color;
    vec2 _texcoord;
};

struct UIInstanceData {
    vec4 _ui_texcoord;
    vec2 _ui_pos;
    vec2 _ui_size;
    uint _ui_color;
    float _ui_round;
    float _ui_border;
    uint _ui_border_color;
    uint _ui_render_flags;
    uint _reserved0;
    uint _reserved1;
    uint _reserved2;
};

layout(binding = 0) uniform sampler2D texture_font;
layout(binding = 1) uniform sampler2D texture_normal;
layout(binding = 2) buffer UIInstanceDataBuffer
{
    UIInstanceData ui_instance_data[MAX_UI_INSTANCE_COUNT];
};

layout( push_constant ) uniform PushConstant_RenderUI
{
    vec2 _inv_canvas_size;
    uint _reserved0;
    uint _reserved1;
} pushConstant;