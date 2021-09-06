#version 460 core
layout(local_size_x = 1, local_size_y = 1) in;
layout(rgba16f, binding = 0) uniform image2D output_img;

void main() {
  // base pixel colour for image
  vec4 pixel = vec4(1.0, 1.0, 1.0, 1.0);
  // get index in global work group i.e x,y position
  ivec2 pixel_coords = ivec2(gl_GlobalInvocationID.xy);
  
  // output to a specific pixel in the image
  imageStore(output_img, pixel_coords, pixel);
}