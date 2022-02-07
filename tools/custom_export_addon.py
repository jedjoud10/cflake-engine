import bpy
import bmesh
import mathutils
from math import radians
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
	edges_to_split = list(filter(filter_edges, tempmesh.edges))	
	bmesh.ops.split_edges(tempmesh, edges = edges_to_split)
	tempmesh.to_mesh(mesh)
	tempmesh.free()
	f = open(filepath, 'w', encoding='utf-8')
	f.write("#Object Name: " + active_object.name + "\n")   	 
	
	# Now loop for every vertex / triangle and write it to the file
	vertex_dict = {}
	vertex_map = {}
	mesh.calc_tangents()
	rotmat = mathutils.Matrix.Rotation(radians(-90), 4, 'X')
	rotmat2 = mathutils.Matrix.Rotation(radians(0), 2, 'X')
	for i, loop in enumerate(mesh.loops):   	   
		vertex = mesh.vertices[loop.vertex_index].co
		
		vertex_new_new = mathutils.Vector((vertex[0], vertex[1], vertex[2]))
		vertex_new_new.rotate(rotmat)
		vertex_new = [0, 0, 0]
		vertex_new[0] = vertex_new_new.x
		vertex_new[1] = vertex_new_new.y
		vertex_new[2] = vertex_new_new.z
		vertex_new = [round(x, 3) for x in vertex_new]
		
		normal = mesh.vertices[loop.vertex_index].normal		
		normal_new_new = mathutils.Vector((normal[0], normal[1], normal[2]))
		normal_new_new.rotate(rotmat)
		normal_new = [0, 0, 0]
		normal_new[0] = normal_new_new.x
		normal_new[1] = normal_new_new.y
		normal_new[2] = normal_new_new.z
		new_normal_new = [int(round(x * 127)) for x in normal_new]
		
		tangent = loop.tangent
		tangent_new_new = mathutils.Vector((tangent[0], tangent[1], tangent[2]))		
		tangent_new_new.rotate(rotmat)
		tangent_new = [0, 0, 0]
		tangent_new[0] = tangent_new_new.x
		tangent_new[1] = tangent_new_new.y
		tangent_new[2] = tangent_new_new.z
		new_tangent_new = [int(round(x * 127)) for x in tangent_new]
		
		uv = mesh.uv_layers.active.data[loop.index].uv		
		new_uv_new = [int(round(x * 255)) for x in uv]
		bitangent_sign = int(round(loop.bitangent_sign))
		vertex_tuple = (vertex_new[0], vertex_new[1], vertex_new[2], normal_new[0], normal_new[1], normal_new[2], new_uv_new[0], new_uv_new[1])
		if not (vertex_tuple in vertex_dict):
			f.write(f'v {vertex_new[0]}/{vertex_new[1]}/{vertex_new[2]}\n')   
			f.write(f'n {new_normal_new[0]}/{new_normal_new[1]}/{new_normal_new[2]}\n')   
			f.write(f't {new_tangent_new[0]}/{new_tangent_new[1]}/{new_tangent_new[2]}/{bitangent_sign}\n')
			f.write(f'u {new_uv_new[0]}/{new_uv_new[1]}\n')   
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
		name="Rigged",
		description="Should we also export the Skeletal rig of this object?",
		default=False,
	)
	def execute(self, context):
		return write_some_data(context, self.filepath, self.skeletal_animation)


# Only needed if you want to add into a dynamic menu
def menu_func_export(self, context):
	self.layout.operator(ExportSomeData.bl_idname, text="cFlake Engine (.mdl3d)")


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
