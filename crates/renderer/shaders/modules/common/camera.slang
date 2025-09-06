module "camera";


public struct CameraUniform {
    public float4 view_pos;
    public float4x4 view;
    public float4x4 view_proj;
    public float4x4 inv_proj;
    public float4x4 inv_view;
}
