#version 460 core
layout (local_size_x = 256, local_size_y = 1, local_size_z = 1) in;

// Spec constants for allocation counts
layout (constant_id = 0) const uint sub_allocations = 1;
layout (constant_id = 1) const uint chunks_per_allocation = 1;

// Sub allocation chunk indices
layout(std430, set = 0, binding = 0) buffer SubAllocationChunkIndices {
    uint[sub_allocations] data;
} indices;

// Chunk indices that we must get rid of
layout(std430, set = 0, binding = 0) buffer ChunkIndicesToRemove {
    uint[chunks_per_allocation] data;
} removed;

void main() {
    for(int i = 0; i < chunks_per_allocation; i++) {
        uint chunk_index_to_remove = removed.data[i];
        uint current = indices.data[gl_GlobalInvocationID.x];
        if(chunk_index_to_remove == current) {
            indices.data[gl_GlobalInvocationID.x] = uint(-1);
        }
    }
}