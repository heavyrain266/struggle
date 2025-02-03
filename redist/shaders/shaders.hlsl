struct PixelInput
{
    float4 position : SV_Position;
    float4 color : COLOR;
};

PixelInput vs_main(float4 position : POSITION, float4 color : COLOR)
{
    PixelInput result;

    result.position = position;
    result.color = color;

    return result;
}

float4 ps_main(PixelInput input) : SV_Target
{
    return input.color;
}
