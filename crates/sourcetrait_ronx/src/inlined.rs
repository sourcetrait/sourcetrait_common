//! Inline include() functionality for RON data files.
//! 
//! SAFETY: This currently doesn't detect circular dependencies.
//! 
//! The [InlinedRon] enum is used as a container for types that can be
//! inline included from Ron files using something like:
//! ```ron,ignore
//! Foo(
//!     bar: true,
//!     my_field: include("mymodule", "my/path/to.ron"),
//! )
//! ```
//! 
//! The module argument to include() can be ommitted.
//! 
//! Use the derive macro and helpers to designate types and fields that can
//! be resolved inline. E.g:
//! ```rust,ignore
//! #[derive(serde::Deserialize, serde::Serialize, RonX)]
//! #[ronx(inlined)]
//! struct Foo {
//!     bar: bool,
//!     // OtherType would need to have a struct-level RonX derive as well
//!     #[ronx(inlined)]
//!     my_field: InlinedRon<OtherType>,
//! }
//! ```
//! 
//! Use the [implement_inlined_ron()] macro to do this outside of the type
//! declaration.
//! 
//! [InlinedRon] will switch values from Include to Included(T), containing the value
//! read from the filesystem, once resolved by the deserializer. Importing is
//! optional and non-imported data is represented by Actual(T).
//! 
//! When serialized, the serializer will treat Included(T) as Include (they
//! both contain path information), so that read/writes are round-trip compatible.
//! Writing included data will need to be done directly against that value.
//! 
use crate::*;

/// Represents data that can be optionally included from somewhere else on the
/// file system.
/// 
/// Refer to the [crate::inlined] module documentation for more information.
pub enum InlinedRon<T> {
    /// Specifies a file and (optional) module to read data the to/from
    Include(IncludeRon),
    /// An include file/module along with the data for it
    Included(RonIncluded<T>),
    /// Actual data not loaded from an include
    Actual(T),
    Unresolved(ron::Value),
}

/// Represents a module and path to read/write data from/to 
#[derive(Debug, Clone, PartialEq)]
pub struct IncludeRon(
    pub Option<String>,
    pub PathBuf
);

pub struct RonIncluded<T>(
    pub Option<String>,
    pub PathBuf,
    pub T
);

use serde::ser::{SerializeTuple, SerializeTupleStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use std::fmt::{self, Debug};
use std::path::PathBuf;
use std::marker::PhantomData;

impl<T: Serialize> Serialize for InlinedRon<T> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTupleStruct;
        
        match self {
            InlinedRon::Include(IncludeRon(module, path)) => {
                if serializer.is_human_readable() {
                    match module {
                        Some(m) => {
                            let mut tuple = serializer.serialize_tuple_struct("include", 2)?;
                            tuple.serialize_field(m)?;
                            tuple.serialize_field(path)?;
                            tuple.end()
                        }
                        None => {
                            path.serialize(serializer)
                        }
                    }
                } else {
                    let mut tuple = serializer.serialize_tuple(2)?;
                    tuple.serialize_element(module)?;
                    tuple.serialize_element(path)?;
                    tuple.end()
                }
            }
            InlinedRon::Included(RonIncluded(module, path, _)) => {
                if serializer.is_human_readable() {
                    match module {
                        Some(m) => {
                            let mut tuple = serializer.serialize_tuple_struct("include", 2)?;
                            tuple.serialize_field(m)?;
                            tuple.serialize_field(path)?;
                            tuple.end()
                        }
                        None => {
                            let mut tuple = serializer.serialize_tuple_struct("include", 2)?;
                            tuple.serialize_field(path)?;
                            tuple.end()
                        }
                    }
                } else {
                    let mut tuple = serializer.serialize_tuple(2)?;
                    tuple.serialize_element(module)?;
                    tuple.serialize_element(path)?;
                    tuple.end()
                }
            }
            InlinedRon::Actual(value) => {
                value.serialize(serializer)
            }
            InlinedRon::Unresolved(_) => {
                Err(serde::ser::Error::custom(
                    "Cannot serialize unresolved InlinedRon variant"
                ))
            }
        }
    }
}

impl<'de, T> Deserialize<'de> for InlinedRon<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct InlinedRonVisitor<T>(PhantomData<T>);
        
        impl<'de, T> Visitor<'de> for InlinedRonVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = InlinedRon<T>;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an include specification, included data, or actual data")
            }
            
            fn visit_enum<A>(self, data: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (variant, variant_access) = data.variant::<String>()?;
                
                if variant == "include" {
                    // for each variant
                    struct IncludeVariantSeed;
                    
                    impl<'de> DeserializeSeed<'de> for IncludeVariantSeed {
                        type Value = IncludeRon;
                        
                        fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
                        where
                            D: Deserializer<'de>,
                        {
                            struct IncludeVariantVisitor;
                            
                            impl<'de> Visitor<'de> for IncludeVariantVisitor {
                                type Value = IncludeRon;
                                
                                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                    formatter.write_str("Include arguments")
                                }
                                
                                fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
                                where
                                    A: SeqAccess<'de>,
                                {
                                    let first = seq.next_element::<serde_value::Value>()?;
                                    let second = seq.next_element::<serde_value::Value>()?;
                                    
                                    match (first, second) {
                                        // 1 arg: path
                                        (Some(first_val), None) => {
                                            let path = match first_val {
                                                serde_value::Value::String(s) => PathBuf::from(s),
                                                _ => PathBuf::deserialize(first_val)
                                                    .map_err(de::Error::custom)?,
                                            };
                                            Ok(IncludeRon(None, path))
                                        }
                                        
                                        // 2 args: module and path
                                        (Some(first_val), Some(second_val)) => {
                                            let module = match first_val {
                                                serde_value::Value::String(s) => Some(s),
                                                serde_value::Value::Unit => None,
                                                _ => return Err(de::Error::custom("expected module string or None")),
                                            };
                                            
                                            let path = match second_val {
                                                serde_value::Value::String(s) => PathBuf::from(s),
                                                _ => PathBuf::deserialize(second_val)
                                                    .map_err(de::Error::custom)?,
                                            };
                                            
                                            Ok(IncludeRon(module, path))
                                        }
                                        
                                        _ => Err(de::Error::custom("invalid Include arguments"))
                                    }
                                }
                            }
                            
                            deserializer.deserialize_seq(IncludeVariantVisitor)
                        }
                    }
                    
                    let include = variant_access.newtype_variant_seed(IncludeVariantSeed)?;
                    Ok(InlinedRon::Include(include))
                    
                } else if variant == "Included" {
                    struct IncludedVariantSeed<T>(PhantomData<T>);
                    
                    impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for IncludedVariantSeed<T> {
                        type Value = RonIncluded<T>;
                        
                        fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
                        where
                            D: Deserializer<'de>,
                        {
                            struct IncludedVariantVisitor<T>(PhantomData<T>);
                            
                            impl<'de, T: Deserialize<'de>> Visitor<'de> for IncludedVariantVisitor<T> {
                                type Value = RonIncluded<T>;
                                
                                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                    formatter.write_str("Included arguments")
                                }
                                
                                fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
                                where
                                    A: SeqAccess<'de>,
                                {
                                    let module = seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                                    let path = seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                                    let data = seq.next_element()?
                                        .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                                    
                                    Ok(RonIncluded(module, path, data))
                                }
                            }
                            
                            deserializer.deserialize_seq(IncludedVariantVisitor(PhantomData))
                        }
                    }
                    
                    let included = variant_access.newtype_variant_seed(IncludedVariantSeed(PhantomData))?;
                    Ok(InlinedRon::Included(included))
                    
                } else {
                    // might be actual data
                    Err(de::Error::unknown_variant(&variant, &["include", "Included"]))
                }
            }           
            
            // untagged Include
            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                // try to parse as Include first
                let first = seq.next_element::<serde_value::Value>()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                
                match first {
                    serde_value::Value::String(s) => {
                        // could be module name or path
                        if let Some(second) = seq.next_element::<serde_value::Value>()? {
                            // two elements: first is module, second is path
                            let path = PathBuf::deserialize(second)
                                .map_err(de::Error::custom)?;
                            Ok(InlinedRon::Include(IncludeRon(Some(s), path)))
                        } else {
                            // single element: it's the path
                            Ok(InlinedRon::Include(IncludeRon(None, PathBuf::from(s))))
                        }
                    }
                    serde_value::Value::Unit => {
                        // first element is None, get the path
                        let path = seq.next_element::<PathBuf>()?
                            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                        Ok(InlinedRon::Include(IncludeRon(None, path)))
                    }
                    _ => {
                        // try to reconstruct and deserialize as T
                        let mut values = vec![first];
                        while let Some(elem) = seq.next_element::<serde_value::Value>()? {
                            values.push(elem);
                        }
                        
                        let value = if values.len() == 1 {
                            values.into_iter().next().unwrap()
                        } else {
                            serde_value::Value::Seq(values)
                        };
                        
                        T::deserialize(value)
                            .map(InlinedRon::Actual)
                            .map_err(de::Error::custom)
                    }
                }
            }
            
            // handle single string (path only)
            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(InlinedRon::Include(IncludeRon(None, PathBuf::from(value))))
            }
            
            // handle map for Included variant or complex Actual types
            fn visit_map<M>(self, map: M) -> std::result::Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let value = serde_value::Value::deserialize(
                    de::value::MapAccessDeserializer::new(map)
                )?;
                
                // check if this looks like an Included
                if let serde_value::Value::Map(ref m) = value {
                    if m.len() == 3 {
                        let has_module = m.contains_key(&serde_value::Value::String("0".to_string()));
                        let has_path = m.contains_key(&serde_value::Value::String("1".to_string()));
                        let has_data = m.contains_key(&serde_value::Value::String("2".to_string()));
                        
                        if has_module && has_path && has_data {
                            match RonIncluded::<T>::deserialize(value.clone()) {
                                Ok(included) => return Ok(InlinedRon::Included(included)),
                                Err(_) => { /* FALLTHRU: try as Actual */ }
                            }
                        }
                    }
                }
                
                // try as Actual
                T::deserialize(value)
                    .map(InlinedRon::Actual)
                    .map_err(de::Error::custom)
            }
        }
        
        deserializer.deserialize_any(InlinedRonVisitor(PhantomData))
    }
}

impl Serialize for IncludeRon {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;
        
        if serializer.is_human_readable() {
            let num_fields = match self.0.is_some() { true => 2, false => 1 };
            let mut tuple = serializer.serialize_tuple_struct("include", num_fields)?;
            if num_fields == 2 {
                tuple.serialize_field(&self.0)?;
            }
            
            tuple.serialize_field(&self.1)?;
            tuple.end()
        } else {
            let mut tuple = serializer.serialize_tuple(2)?;
            tuple.serialize_element(&self.0)?;
            tuple.serialize_element(&self.1)?;
            tuple.end()
        }
    }
}

impl<'de> Deserialize<'de> for IncludeRon {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IncludeRonVisitor;
        
        impl<'de> Visitor<'de> for IncludeRonVisitor {
            type Value = IncludeRon;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("include specification as (Option<String>, PathBuf) or just PathBuf")
            }
            
            // handle just a path string
            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(IncludeRon(None, PathBuf::from(value)))
            }
            
            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let first = seq.next_element::<Option<String>>()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let second = seq.next_element::<PathBuf>()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(IncludeRon(first, second))
            }
            
            // handle newtype struct specifically (for Include(...) syntax)
            fn visit_newtype_struct<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                // handles Include(...) specifically
                deserializer.deserialize_any(self)
            }
        }
        
        deserializer.deserialize_any(IncludeRonVisitor)
    }
}

impl<T: Serialize> Serialize for RonIncluded<T> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTupleStruct;
        // only serialize the first two fields, skip T
        
        let num_fields = match self.0.is_some() { true => 2, false => 1 };
        if serializer.is_human_readable() {
            let mut tuple = serializer.serialize_tuple_struct("include", num_fields)?;
            if num_fields == 2 {
                tuple.serialize_field(&self.0)?;
            }
            
            tuple.serialize_field(&self.1)?;
            tuple.end()
        } else {
            let mut tuple = serializer.serialize_tuple(num_fields)?;
            if num_fields == 2 {
                tuple.serialize_element(&self.0)?;
            }
            
            tuple.serialize_element(&self.1)?;
            tuple.end()
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for RonIncluded<T> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RonIncludedVisitor<T>(PhantomData<T>);
        
        impl<'de, T: Deserialize<'de>> Visitor<'de> for RonIncludedVisitor<T> {
            type Value = RonIncluded<T>;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("included data with module, path, and data")
            }
            
            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let module = seq.next_element::<Option<String>>()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let path = seq.next_element::<PathBuf>()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let data = seq.next_element::<T>()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                
                Ok(RonIncluded(module, path, data))
            }
        }
        
        deserializer.deserialize_tuple_struct("include", 3, RonIncludedVisitor(PhantomData))
    }
}

impl<T> InlinedRon<T> {
    pub const fn is_actual(&self) -> bool {
        matches!(self, InlinedRon::Actual(_))
    }

    pub const fn is_include(&self) -> bool {
        matches!(self, InlinedRon::Include{..})
    }
    
    pub const fn is_included(&self) -> bool {
        matches!(self, InlinedRon::Included(_))
    }
    
    pub const fn actual(&self) -> Option<&T> {
        match self {
            InlinedRon::Actual(value) => Some(value),
            _ => None,
        }
    }
    
    pub const fn included(&self) -> Option<&T> {
        match self {
            InlinedRon::Included(RonIncluded(_, _, value)) => Some(value),
            _ => None,
        }
    }
    
    pub const fn inlined(&self) -> Option<&T> {
        match self {
            InlinedRon::Actual(value) => Some(value),
            InlinedRon::Included(RonIncluded(_, _, value)) => Some(value),
            _ => None,
        }
    }
    
    pub fn take_inlined(self) -> Option<T> {
        match self {
            InlinedRon::Actual(value) => Some(value),
            InlinedRon::Included(RonIncluded(_, _, value)) => Some(value),
            _ => None,
        }
    }
    
    pub const unsafe fn expect_inlined(&self) -> &T {
        self.inlined().expect("inlined ron value to be resolved")
    }
}

/// Tags a type as resolvable as well as calls resolvers for each field or
/// variant that needs resolutioin.
pub trait InlinedRonResolver {
    fn resolve_inlined_ron(&mut self, state: &mut InlinedRonState, current: &Path, config: &InlinedRonConfig) -> Result<()>;
}

/// Include dirs are named and can be specified in Ron along with a
/// relative filepath.
/// 
/// The default include dir, if not provided, is named "crate".
/// 
/// Attempting to access files outside of the include dirs list will throw
/// an error.
#[derive(Debug)]
pub struct InlinedRonConfig {
    pub include_dirs: InlinedRonIncludeDirs,
    pub resolve_includes: bool,
}

impl InlinedRonConfig {
    pub const DEFAULT_MODULE: &'static str = "crate";
    
    pub fn new(default_dir: PathBuf) -> Self {
        Self {
            include_dirs: InlinedRonIncludeDirs::new(Self::DEFAULT_MODULE.into(), default_dir, None),
            resolve_includes: true,
        }
    }
    
    pub fn new_dirs(default_dir: PathBuf, dirs: HashMap<String, PathBuf>) -> Self {
        Self {
            include_dirs: InlinedRonIncludeDirs::new(Self::DEFAULT_MODULE.into(), default_dir, Some(dirs)),
            resolve_includes: true,
        }
    }
}

/// All paths are expected to be canonical
/// First path in map is always the default
#[derive(Debug, Clone)]
pub struct InlinedRonIncludeDirs(LinkedHashMap<String, PathBuf>);

pub struct CanonicalIncludePath<'a> {
    pub include_dir_name: &'a str,
    pub include_dir: &'a Path,
    pub filepath: PathBuf,
}

impl<'a> CanonicalIncludePath<'a> {
    fn new(include_dir_name: &'a str, include_dir: &'a Path, filepath: PathBuf) -> Self {
        CanonicalIncludePath {
            include_dir_name,
            include_dir,
            filepath,
        }
    }
}

impl InlinedRonIncludeDirs {
    pub fn new(
        default_dir_name: String,
        default_dir: PathBuf,
        dirs: Option<HashMap<String, PathBuf>>
    ) -> Self {
        let mut map = LinkedHashMap::with_capacity(
            dirs.as_ref().map(|d| 1+d.len()).unwrap_or(1)
        );
        
        map.insert(default_dir_name, default_dir);
        
        if let Some(dirs) = dirs {
            map.extend(dirs);
        }
        
        Self(map)
    }
    
    pub fn default_dir(&self) -> (&str, &Path) {
        self.0.front().map(|(k,v)| (k.as_ref(), v.as_path())).expect("front")
    }
    
    pub fn dir(&self, name: &str) -> Option<(&str, &Path)> {
        self.0.get_key_value(name).map(|(name, path)| (name.as_ref(), path.as_path()))
    }
    
    /// Returns (include_dir , canonical filepath )
    pub fn canonical<'a>(&'a self, module: Option<&str>, filepath: &Path) -> Result<CanonicalIncludePath<'a>> {
        let (include_dir_name, include_dir) = module 
            .and_then(|module| self.dir(module))
            .unwrap_or_else(|| self.default_dir());
        
        let ron_filepath = if filepath.is_relative() {
            include_dir.join(&filepath).canonicalize()
        } else {
            filepath.canonicalize()
        };
        
        ron_filepath
            .map(|p| CanonicalIncludePath::new(include_dir_name, include_dir, p))
            .map_err(|e| Error::Io(FsErrMsg::ReadFile(RON), filepath.to_path_buf(), e))
    }
}

pub struct ReadRonValue<'a, T>(CanonicalIncludePath<'a>, T)
where
    T: for<'de> Deserialize<'de> + FromInlinedRon + InlinedRonResolver;

fn read_ron<'a, T>(
    state: &mut InlinedRonState,
    current: &Path,
    module: Option<&str>,
    filepath: &Path,
    config: &'a InlinedRonConfig
) -> Result<ReadRonValue<'a, T>>
where
    T: for<'de> Deserialize<'de> + FromInlinedRon + InlinedRonResolver
{
    let include_path = config.include_dirs.canonical(module, filepath)?;
    
    if include_path.filepath.strip_prefix(include_path.include_dir).is_err() {
        return Err(Error::Io(FsErrMsg::AccessFile(RON), include_path.filepath.into(),
            io::Error::new(io::ErrorKind::PermissionDenied, ""))
        );
    }
    
    if !include_path.filepath.extension().is_some_and(|ext| ext == LIL_RON) {
        return Err(Error::Io(FsErrMsg::AccessFile(RON), include_path.filepath.into(),
            io::Error::new(io::ErrorKind::PermissionDenied, "Not a RON file"))
        );
    }
        
    // circular dependency check
    state.add_dependency(current, include_path.filepath.clone())?;
    
    let value = T::from_ron_file(&include_path.filepath)?;
    Ok(ReadRonValue(include_path, value))
}

impl<T> InlinedRonResolver for InlinedRon<T>
where
    T: FromRon + InlinedRonResolver + for<'de> Deserialize<'de> + FromInlinedRon
{
    fn resolve_inlined_ron(&mut self, state: &mut InlinedRonState, current: &Path, config: &InlinedRonConfig) -> Result<()> {
        *self = match std::mem::replace(self, InlinedRon::Unresolved(ron::Value::Unit)) {
            InlinedRon::Include(IncludeRon(module, path)) if config.resolve_includes => {
                let ReadRonValue::<T>(include_path, mut value)
                    = read_ron(state, current, module.as_deref(), &path, config)?;
                
                value.resolve_inlined_ron(state, &include_path.filepath, config)?;
                InlinedRon::Included(RonIncluded(module, path, value))
            }
            InlinedRon::Actual(mut value) => {
                value.resolve_inlined_ron(state, &current, config)?;
                InlinedRon::Actual(value)
            }
            InlinedRon::Included(RonIncluded(module, path, mut value)) => {
                let filepath = config.include_dirs.canonical(module.as_deref(), &path)?.filepath;
                value.resolve_inlined_ron(state, &filepath, config)?;
                InlinedRon::Included(RonIncluded(module, path, value))
            }
            other => other,
        };
        Ok(())
    }
}

impl<T> InlinedRonResolver for Vec<T>
where
    T: InlinedRonResolver 
{
    fn resolve_inlined_ron(&mut self, state: &mut InlinedRonState, current: &Path, config: &InlinedRonConfig) -> Result<()> {
        for item in self.iter_mut() {
            item.resolve_inlined_ron(state, current, config)?;
        }
        
        Ok(())
    }
}

impl<T> InlinedRonResolver for Option<T>
where
    T: InlinedRonResolver
{
    fn resolve_inlined_ron(&mut self, state: &mut InlinedRonState, current: &Path, config: &InlinedRonConfig) -> Result<()> {
        if let Some(inner) = self {
            inner.resolve_inlined_ron(state, current, config)?;
        }
        
        Ok(())
    }
}

impl InlinedRonResolver for String {
    fn resolve_inlined_ron(&mut self, _: &mut InlinedRonState, _: &Path, _: &InlinedRonConfig) -> Result<()> {
        Ok(())
    }
}

impl InlinedRonResolver for u128 {
    fn resolve_inlined_ron(&mut self, _: &mut InlinedRonState, _: &Path, _: &InlinedRonConfig) -> Result<()> {
        Ok(())
    }
}

impl InlinedRonResolver for u64 {
    fn resolve_inlined_ron(&mut self, _: &mut InlinedRonState, _: &Path, _: &InlinedRonConfig) -> Result<()> {
        Ok(())
    }
}

impl InlinedRonResolver for u32 {
    fn resolve_inlined_ron(&mut self, _: &mut InlinedRonState, _: &Path, _: &InlinedRonConfig) -> Result<()> {
        Ok(())
    }
}

impl InlinedRonResolver for u16 {
    fn resolve_inlined_ron(&mut self, _: &mut InlinedRonState, _: &Path, _: &InlinedRonConfig) -> Result<()> {
        Ok(())
    }
}

impl InlinedRonResolver for u8 {
    fn resolve_inlined_ron(&mut self, _: &mut InlinedRonState, _: &Path, _: &InlinedRonConfig) -> Result<()> {
        Ok(())
    }
}

impl InlinedRonResolver for f64 {
    fn resolve_inlined_ron(&mut self, _: &mut InlinedRonState, _: &Path, _: &InlinedRonConfig) -> Result<()> {
        Ok(())
    }
}

impl InlinedRonResolver for f32 {
    fn resolve_inlined_ron(&mut self, _: &mut InlinedRonState, _: &Path, _: &InlinedRonConfig) -> Result<()> {
        Ok(())
    }
}

impl InlinedRonResolver for bool {
    fn resolve_inlined_ron(&mut self, _: &mut InlinedRonState, _: &Path, _: &InlinedRonConfig) -> Result<()> {
        Ok(())
    }
}

impl<T: PartialEq> PartialEq for InlinedRon<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Actual(a), Self::Actual(b)) => a == b,
            (Self::Include(a), Self::Include(b)) => a == b,
            (Self::Included(a), Self::Included(b)) => a == b,
            _ => false,
        }
    }
}

impl<T: Clone> Clone for InlinedRon<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Actual(v) => Self::Actual(v.clone()),
            Self::Include(v) => Self::Include(v.clone()),
            Self::Included(v) => Self::Included(v.clone()),
            Self::Unresolved(v) => Self::Unresolved(v.clone()),
        }
    }
}

impl<T: Debug> Debug for InlinedRon<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Actual(v) => f.debug_tuple("Actual").field(v).finish(),
            Self::Include(v) => f.debug_tuple("Include").field(v).finish(),
            Self::Included(v) => f.debug_tuple("Included").field(v).finish(),
            Self::Unresolved(v) => f.debug_tuple("Unresolved").field(v).finish(),
        }
    }
}

impl<T: Clone> Clone for RonIncluded<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), self.2.clone())
    }
}

impl<T: PartialEq> PartialEq for RonIncluded<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}

impl<T: Debug> Debug for RonIncluded<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Included")
            .field(&self.0)
            .field(&self.1)
            .field(&self.2)
            .finish()
    }
}

pub trait FromInlinedRon: FromRon + InlinedRonResolver {
    fn from_inlined_ron(ron: &str, config: &InlinedRonConfig) -> Result<Self>
    where
        Self: for<'de> serde::de::Deserialize<'de>
    {
        let mut value = Self::from_ron(ron)?;
        let mut state = InlinedRonState::new();
        let current = PathBuf::from("<string>");
        value.resolve_inlined_ron(&mut state, &current, &config)?;
        Ok(value)
    }
    
    /// Deserializes a type that contains [InlinedRon] fields from their specified 
    /// files based on the information in each [RonInclude], using [RonIncludeDirs]
    /// as the basis for relative paths. Each include file is read, parsed, and resolved.
    fn from_inlined_ron_file(ron_filepath: &Path, config: &InlinedRonConfig) -> Result<Self>
    where
        Self: for<'de> serde::de::Deserialize<'de>
    {
        let mut value = Self::from_ron_file(ron_filepath)?;
        let mut state = InlinedRonState::new();
        value.resolve_inlined_ron(&mut state, &ron_filepath, &config)?;
        Ok(value)
    }
}

pub trait ToInlinedRon: ToRon + InlinedRonResolver {
    /// This will serialize InlineRon::Included as InlinedRon::Include
    fn to_inlined_ron(&self, _config: &InlinedRonConfig) -> Result<String>
    where
        Self: serde::ser::Serialize
    {
        Self::to_ron(self)
    }
    
    /// Serializes a type that contains [InlinedRon] fields to their specified 
    /// files based on the information in each [RonInclude], using [RonIncludeDirs]
    /// as the basis for relative paths. Each include is parsed into string and written to file.
    /// 
    /// This will serialize InlineRon::Included as InlinedRon::Include
    fn to_inlined_ron_file(&self, ron_filepath: &Path, _config: &InlinedRonConfig) -> Result<()>
    where
        Self: serde::ser::Serialize
    {
        Self::to_ron_file(self, ron_filepath)
    }
}

pub struct InlinedRonState {
    path_graph: PathGraph
}

impl InlinedRonState {
    pub fn new() -> Self {
        Self {
            path_graph: PathGraph::new()
        }
    }
    
    pub fn path_graph(&self) -> &PathGraph {
        &self.path_graph
    }
    
    pub fn path_graph_mut(&mut self) -> &mut PathGraph {
        &mut self.path_graph
    }
    
    pub fn add_dependency(&mut self, from: &Path, to: PathBuf) -> Result<()> {
        self.path_graph.add_dependency(from, &to)
            .map_err(|e| Error::ResolveRON(e))
    }
}