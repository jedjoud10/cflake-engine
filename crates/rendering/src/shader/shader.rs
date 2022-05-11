use assets::Asset;


// Specifies the file paths for the vertex and fragment sources that make up a specific shader
pub struct Settings<'a> {
    vertex: &'a str,
    fragment: &'a str,
    geometry: Option<&'a str>,
    tesselation: Option<&'a str>,
}

impl<'a> Settings<'a> {
    // Create a new shader sources paths that we will load later
    fn new(vertex_source_path: &'a str, fragment_source_path: &'a str, geometry_source_path: Option<&'a str>, tesselation_source_path: Option<&'a str>) -> Self {
        Self {
            vertex: vertex_source_path,
            fragment: fragment_source_path,
            geometry: geometry_source_path,
            tesselation: tesselation_source_path,
        }
    }
}

// A shader that will render our objects onto the screen
// This will make use of two shader programs, the vertex programs, and fragment program
pub struct Shader {
    vertex: super::Source,
    fragment: super::Source,
    geometry: Option<super::Source>,
    tesselation: Option<super::Source>,
}


impl<'a> Asset<'a> for Shader {
    type OptArgs = Settings<'a>;

    fn is_valid(meta: assets::metadata::AssetMetadata) -> bool {
        todo!()
    }
    
    fn try_load_with<'l>(loader: &assets::loader::AssetLoader, path: &str, args: Self::OptArgs) -> Option<Self>
    where
            Self: Sized, {
        // Try to load each shader source
        let vertex = String::try_load(loader, args.vertex).expect("Vertex source could not be loaded");
        let fragment = String::try_load(loader, args.fragment).expect("Fragment source could not be loaded");
        let geometry = { String::try_load(loader, args.geometry?) };
        let tesselation = { String::try_load(loader, args.tesselation?) };
    }

    unsafe fn deserialize(bytes: &[u8], args: Self::OptArgs) -> Option<Self>
    where
        Self: Sized {
        todo!()
    }
}