// General info
uniform float _time;
uniform float _delta;
uniform ivec2 _resolution;
uniform vec2 _nf_planes;

// Convert depth to linear depth
float to_linear_depth(float odepth) {
    return (_nf_planes.x * odepth) / (_nf_planes.y - odepth * (_nf_planes.y - _nf_planes.x));	
}

// Convert linear depth to world depth
float to_world_depth(float ldepth) {
    return _nf_planes.x + ldepth * _nf_planes.y; 
}