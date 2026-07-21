use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
};
use thiserror::Error;

#[macro_export]
macro_rules! shaders_file {
    ($as_mod:ident) => {
        mod $as_mod {
            include!(concat!(env!("OUT_DIR"), "/shaders/shaders.rs"));
        }
    };
}

mod rustfile {
    use crate::build_tools::ShaderBuilderError;
    use std::fmt::Write as _;
    use std::path::{Path, PathBuf};

    pub fn generate_shader_include_file(
        shaders: &[(String, PathBuf)],
        target_file: &Path,
    ) -> Result<(), ShaderBuilderError> {
        let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);

        let mut out = String::new();

        out.push_str("// @generated\n// by build.rs — do not edit by hand\n\n");

        let mut const_names = Vec::with_capacity(shaders.len());

        for (name, path) in shaders {
            let const_name = to_const_ident(name);
            let rel_path = path.strip_prefix(&out_dir).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "shader path {} is not inside out_dir {}",
                        path.display(),
                        out_dir.display()
                    ),
                )
            })?;
            let path_str = escape_path(rel_path);

            writeln!(
                out,
                "pub static {const_name}: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{path_str}\"));"
            )
            .expect("Writing to String cannot fail");

            const_names.push((name.clone(), const_name));
        }

        out.push('\n');

        out.push_str("pub static SHADERS: &[(&str, &[u8])] = &[\n");
        for (name, const_name) in const_names {
            writeln!(out, "  (\"{name}\", {const_name}),").expect("Writing to String cannot fail");
        }
        out.push_str("];\n");

        std::fs::write(target_file, out)?;

        Ok(())
    }

    fn to_const_ident(name: &str) -> String {
        let mut ident: String = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .to_uppercase();

        if ident.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            ident.insert(0, '_');
        }

        ident
    }

    fn escape_path(path: &Path) -> String {
        path.to_string_lossy()
            .replace('\\', "/")
            .replace('"', "\\\"")
    }
}

#[derive(Error, Debug)]
pub enum ShaderBuilderError {
    #[error("Encountered error in env var operation: {0}")]
    EnvVarError(#[from] std::env::VarError),

    #[error("Encountered IO error: {0}\n{1}")]
    IOError(std::io::Error, String),

    #[error("Encountered unknown IO error: {0}")]
    UnknownIOError(#[from] std::io::Error),

    #[error("Encountered error in deserializing TOML: {0}")]
    TOMLDeError(#[from] toml::de::Error),

    #[error("Encountered error in deserializing TOML: {0}")]
    TOMLSerError(#[from] toml::ser::Error),

    #[error("Error while compiling shader: {0}")]
    CompilerError(String),
}
impl ShaderBuilderError {
    fn from_io_error(err: std::io::Error, context: String) -> Self {
        Self::IOError(err, context)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ShaderStage {
    Vertex,
    Pixel,
    Compute,
}

#[derive(Serialize, Deserialize)]
pub struct ShaderMetadata {
    file: PathBuf,
    stage: ShaderStage,
    backend: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ShaderBuilderConfig {
    always_rebuild: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ShaderCrateMetadata {
    config: ShaderBuilderConfig,
    shaders: HashMap<String, ShaderMetadata>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ShaderLockFile {
    shaders: HashMap<String, u64>,
}

pub struct ShaderSource {
    source: Vec<u8>,
    stage: ShaderStage,
    backend: String,
}
pub struct ShaderSources {
    config: ShaderBuilderConfig,
    sources: HashMap<String, ShaderSource>,
}

impl ShaderSources {
    /// Read shader metadata for this target crate's shaders
    pub fn read() -> Result<Self, ShaderBuilderError> {
        let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
        let metadata_file = crate_dir.join("shaders.toml");

        println!("cargo:rerun-if-changed={}", metadata_file.display());

        if !metadata_file.exists() {
            return Ok(Self {
                sources: HashMap::default(),
                config: ShaderBuilderConfig::default(),
            });
        }

        let metadata: ShaderCrateMetadata =
            toml::from_str(&std::fs::read_to_string(metadata_file).map_err(|err| {
                ShaderBuilderError::from_io_error(
                    err,
                    "Couldn't read shader metadata file `shaders.toml` in crate root".to_string(),
                )
            })?)?;

        let sources = metadata
            .shaders
            .into_iter()
            .map(|(name, meta)| {
                println!("cargo:rerun-if-changed={}", meta.file.display());

                Ok((
                    name,
                    ShaderSource {
                        source: std::fs::read(&meta.file).map_err(|err| {
                            ShaderBuilderError::from_io_error(
                                err,
                                format!("Couldn't read shader from file '{}'", meta.file.display()),
                            )
                        })?,
                        stage: meta.stage,
                        backend: meta.backend,
                    },
                ))
            })
            .collect::<Result<HashMap<String, ShaderSource>, ShaderBuilderError>>()?;

        Ok(Self {
            sources,
            config: metadata.config,
        })
    }
}

pub trait ShaderCompiler {
    fn backend_name(&self) -> &'static str;

    /// Build shaders that were read using `ShaderCrateMetadara::read()`
    ///
    /// Will skip building if all shaders are already built
    ///
    /// `shaders`: Shaders read by the metadata reader
    fn build_compatible_shaders(&self, shaders: ShaderSources) -> Result<(), ShaderBuilderError> {
        let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
        let lock_file = crate_dir.join("shaders.lock");

        if shaders.sources.is_empty() {
            return Ok(());
        }

        let backend_name = self.backend_name();
        let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
        let build_dir = out_dir
            .parent()
            .and_then(|dir| dir.parent())
            .and_then(|dir| dir.parent())
            .ok_or(ShaderBuilderError::CompilerError(format!(
                "Invalid OUT_DIR path: {}",
                out_dir.display()
            )))?;
        let shaders_dir = build_dir.join("shaders");
        let shaders_out_dir = out_dir.join("shaders");

        let shaders_include_file = shaders_out_dir.join("shaders.rs");

        for dir in [&shaders_dir, &shaders_out_dir] {
            std::fs::create_dir_all(dir).map_err(|err| {
                ShaderBuilderError::IOError(
                    err,
                    format!(
                        "Failed to create temporary shaders directory {}",
                        shaders_dir.display()
                    ),
                )
            })?;
        }

        let mut shaders_lock: ShaderLockFile = if lock_file.exists() {
            toml::from_str(&std::fs::read_to_string(&lock_file)?)?
        } else {
            std::fs::write(&lock_file, "[shaders]")?;
            ShaderLockFile::default()
        };

        let mut all_bins = Vec::default();
        let mut bins = Vec::default();

        for (name, source) in shaders.sources {
            let file = shaders_dir.join(format!("{name}.{backend_name}"));
            let out_file = shaders_out_dir.join(format!("{name}.{backend_name}"));

            if !shaders.config.always_rebuild && out_file.exists() && file.exists() {
                let mut hasher = DefaultHasher::new();
                source.source.hash(&mut hasher);
                let hash = hasher.finish();

                if let Some(old_hash) = shaders_lock.shaders.get(&name)
                    && *old_hash == hash
                {
                    std::fs::copy(file, out_file.clone())?;
                    all_bins.push((name, out_file));
                    continue;
                }
                shaders_lock.shaders.insert(name.clone(), hash);
            }

            if source.backend != backend_name {
                if file.exists() {
                    std::fs::copy(file, out_file.clone())?;
                    all_bins.push((name, out_file));
                }
                continue;
            }

            let bin = self.build_shader(&name, source.stage, source.source)?;

            std::fs::write(&file, bin).map_err(|err| {
                ShaderBuilderError::IOError(
                    err,
                    format!("Failed to write to file {}", file.display()),
                )
            })?;
            std::fs::copy(file.clone(), out_file.clone())?;

            all_bins.push((name.clone(), out_file.clone()));
            bins.push((name, file));
        }

        if bins.is_empty() {
            return Ok(());
        }

        rustfile::generate_shader_include_file(&all_bins, &shaders_include_file)?;

        std::fs::write(lock_file, toml::to_string(&shaders_lock)?)?;

        Ok(())
    }

    fn build_shader(
        &self,
        name: &str,
        stage: ShaderStage,
        source: Vec<u8>,
    ) -> Result<Vec<u8>, ShaderBuilderError>;
}
