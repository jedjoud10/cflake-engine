import bpy
import bmesh
from bpy import context

bl_info = {
    "name": "Custom 3D model exporter",
    "blender": (2, 80, 0),
    "category": "Object",
}

def filter_edges(edge):
    return not edge.smooth

def write_some_data(context, filepath, skeletal_animation):
    active_object = context.active_object
    mesh = active_object.data.copy()
    tempmesh = bmesh.new()
    tempmesh.from_mesh(mesh)
    bmesh.ops.triangulate(tempmesh, faces = tempmesh.faces[:])
    edges_to_split = filter(filter_edges, tempmesh.edges)
    bmesh.ops.split_edges(tempmesh, edges = list(edges_to_split))
    tempmesh.to_mesh(mesh)
    tempmesh.free()
    f = open(filepath, 'w', encoding='utf-8')
    f.write("#Object Name: " + active_object.name + "\n")        
    
    # Now loop for every vertex / triangle and write it to the file
    vertex_dict = {}
    vertex_map = {}
    mesh.calc_tangents()
    for i, loop in enumerate(mesh.loops):             
        vertex = mesh.vertices[loop.vertex_index].co
        vertex_new = [round(x, 3) for x in vertex]
        normal = mesh.vertices[loop.vertex_index].normal
        normal_new = [round(x, 3) for x in normal]
        tangent = loop.tangent
        tangent_new = [round(x, 3) for x in tangent]
        uv = mesh.uv_layers.active.data[loop.index].uv
        uv_new = [round(x, 3) for x in uv]
        vertex_tuple = (vertex_new[0], vertex_new[1], vertex_new[2], normal_new[0], normal_new[1], normal_new[2], uv_new[0], uv_new[1])
        if not (vertex_tuple in vertex_dict):
            f.write(f'v {vertex_new[0]}/{vertex_new[1]}/{vertex_new[2]}\n')   
            f.write(f'n {normal_new[0]}/{normal_new[1]}/{normal_new[2]}\n')   
            f.write(f't {tangent_new[0]}/{tangent_new[1]}/{tangent_new[2]}\n')   
            f.write(f'u {uv_new[0]}/{uv_new[1]}\n')   
            vertex_dict[vertex_tuple] = len(vertex_dict)
            vertex_map[loop.vertex_index] = len(vertex_dict) - 1
        else:
            vertex_map[loop.vertex_index] = vertex_dict[vertex_tuple]
    # Write the triangles
    for polygon in mesh.polygons:
        f.write('i ')
        for index in range(polygon.loop_start, polygon.loop_start + polygon.loop_total):
            uv = mesh.uv_layers.active.data[index].uv
            loop = mesh.loops[index]
            vertex = mesh.vertices[loop.vertex_index].co
            normal = mesh.vertices[loop.vertex_index].normal
            first_slash = "/"
            if index == polygon.loop_start:
                first_slash = ""
            f.write(first_slash + f'{vertex_map[loop.vertex_index]}')  
        f.write('\n') 
    
    f.close()
    return {'FINISHED'}


# ExportHelper is a helper class, defines filename and
# invoke() function which calls the file selector.
from bpy_extras.io_utils import ExportHelper
from bpy.props import StringProperty, BoolProperty, EnumProperty
from bpy.types import Operator


class ExportSomeData(Operator, ExportHelper):
    """Exports models to a file that our game engine can import and pack!"""
    bl_idname = "export_test.some_data"  # important since its how bpy.ops.import_test.some_data is constructed
    bl_label = "Export Model"

    # ExportHelper mixin class uses this
    filename_ext = ".mdl3d"

    filter_glob: StringProperty(
        default="*.mdl3d",
        options={'HIDDEN'},
        maxlen=255,  # Max internal buffer length, longer would be clamped.
    )

    # List of operator properties, the attributes will be assigned
    # to the class instance from the operator settings before calling.
    skeletal_animation: BoolProperty(
        name="Skeletal Animation",
        description="Should we also export the Skeletal animation of this object if it is a Skeleton?",
        default=False,
    )
    def execute(self, context):
        return write_some_data(context, self.filepath, self.skeletal_animation)


# Only needed if you want to add into a dynamic menu
def menu_func_export(self, context):
    self.layout.operator(ExportSomeData.bl_idname, text="Hypoengine3D (.mdl3d)")


def register():
    bpy.utils.register_class(ExportSomeData)
    bpy.types.TOPBAR_MT_file_export.append(menu_func_export)


def unregister():
    bpy.utils.unregister_class(ExportSomeData)
    bpy.types.TOPBAR_MT_file_export.remove(menu_func_export)


if __name__ == "__main__":
    register()

    # test call
    bpy.ops.export_test.some_data('INVOKE_DEFAULT')
