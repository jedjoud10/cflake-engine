// Normal compression algorithms stolen from https://www.shadertoy.com/view/llfcRl
// TODO: Add the rest and document

// Compresses a normalized normal into 16 bits using cubic compression
uint cube_16( in vec3 nor )
{
    vec3 mor; uint  id;
                                    mor = nor.xyz; id = 0u;
    if( abs(nor.y) > abs(mor.x) ) { mor = nor.yzx; id = 1u; }
    if( abs(nor.z) > abs(mor.x) ) { mor = nor.zxy; id = 2u; }
    uint is = (mor.x<0.0)?1u:0u;
    vec2 uv = 0.5 + 0.5*mor.yz/abs(mor.x);
    uvec2 iuv = uvec2(round(uv*vec2(127.0,63.0)));
    return iuv.x | (iuv.y<<7u) | (id<<13u) | (is<<15u);
}

// Decompress a 16 bit compressed cubic normal  
vec3 i_cube_16( uint data )
{
    uvec2 iuv = uvec2( data, data>>7u ) & uvec2(127u,63u);
    vec2 uv = vec2(iuv)*2.0/vec2(127.0,63.0) - 1.0;
    uint is = (data>>15u)&1u;
    vec3 nor = vec3((is==0u)?1.0:-1.0,uv.xy);
    uint id = (data>>13u)&3u;
         if(id==0u) nor = nor.xyz;
    else if(id==1u) nor = nor.zxy;
    else            nor = nor.yzx;
    return normalize(nor);
}