// Get the SDF to the scene
float sdf(vec3 point) {
    return min(point.y, length(point) - 2) + sin(point.x*4);
}

// Calculate lighting from normal
vec3 lighting(vec3 normal) {
    return vec3(normal.y);
}

// Calculate normals based on derivatives
vec3 normal(vec3 point, float epsilon) {
    vec3 left = vec3(epsilon, 0, 0);
    vec3 up = vec3(0, epsilon, 0);
    vec3 forward = vec3(0, 0, epsilon);

    return normalize(vec3(
        sdf(point + left) - sdf(point - left),
        sdf(point + up) - sdf(point - up),
        sdf(point + forward) - sdf(point - forward)
    ));
}