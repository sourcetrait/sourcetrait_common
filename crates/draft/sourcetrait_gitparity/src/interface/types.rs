use std::{
    borrow::Cow, collections::HashMap, fmt::{Debug, Display}, fs, path::Path, str::FromStr};
use chrono::{DateTime, Utc};
use crate::*;

#[derive(Debug, Clone, Default, derive_builder::Builder)]
#[builder(setter(strip_option))]
pub struct GitEnv {
    author_name: Option<String>,
    author_email: Option<String>,
    author_date: Option<DateTime<Utc>>,
    committer_name: Option<String>,
    committer_email: Option<String>,
    committer_date: Option<DateTime<Utc>>
}

impl GitEnv {
    pub fn builder() -> GitEnvBuilder {
        GitEnvBuilder::default()
    }
    
    pub fn author_name(&self) -> Option<&str> {
        self.author_name.as_deref()
    }

    pub fn author_email(&self) -> Option<&str> {
        self.author_email.as_deref()
    }

    pub fn author_date(&self) -> Option<DateTime<Utc>> {
        self.author_date
    }
    
    pub fn author_datestamp(&self) -> Option<String> {
        self.author_date.map(|d| d.to_rfc3339())
    }

    pub fn committer_name(&self) -> Option<&str> {
        self.committer_name.as_deref()
    }

    pub fn committer_email(&self) -> Option<&str> {
        self.committer_email.as_deref()
    }

    pub fn committer_date(&self) -> Option<DateTime<Utc>> {
        self.committer_date
    }
    
    pub fn committer_datestamp(&self) -> Option<String> {
        self.committer_date.map(|d| d.to_rfc3339())
    }
}

impl GitEnv {
    pub fn set_author_name(&mut self, name: String) {
        self.author_name = Some(name);
    }

    pub fn set_author_email(&mut self, email: String) {
        self.author_email = Some(email);
    }

    pub fn set_author_date(&mut self, date: DateTime<Utc>) {
        self.author_date = Some(date);
    }
    
    pub fn set_committer_name(&mut self, name: String) {
        self.committer_name = Some(name);
    }

    pub fn set_committer_email(&mut self, email: String) {
        self.committer_email = Some(email);
    }

    pub fn set_committer_date(&mut self, date: DateTime<Utc>) {
        self.committer_date = Some(date);
    }
    
    pub fn unset_author_name(&mut self) {
        self.author_name = None;
    }

    pub fn unset_author_email(&mut self) {
        self.author_email = None;
    }

    pub fn unset_author_date(&mut self) {
        self.author_date = None;
    }
    
    pub fn unset_committer_name(&mut self) {
        self.committer_name = None;
    }

    pub fn unset_committer_email(&mut self) {
        self.committer_email = None;
    }

    pub fn unset_committer_date(&mut self) {
        self.committer_date = None;
    }
}


#[derive(Clone, PartialEq, Eq)]
pub struct GitOID([u8; 20]);

impl GitOID {
    pub fn new(bytes: [u8; 20]) -> Result<Self> {
        Ok(GitOID(bytes))
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for GitOID {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl FromStr for GitOID {
    type Err = Error;
    
    fn from_str(hex: &str) -> Result<Self> {
        if hex.len() != 40 {
            return Err(Error::FromStr("Git hash must be 40 characters long".to_string()));
        }
        
        let mut bytes = [0u8; 20];
        hex::decode_to_slice(hex, &mut bytes as &mut [u8])
                .map_err(|_| Error::FromStr(format!("Invalid Git hash: {hex}")))?;
        
        Ok(GitOID(bytes))
    }
}

impl TryFrom<&[u8]> for GitOID {
    type Error = Error;
    
    fn try_from(value: &[u8]) -> Result<Self> {
        if value.len() != 20 {
            return Err(Error::FromStr("Git hash must be 20 bytes".to_string()));
        }
        
        let bytes = <[u8; 20]>::try_from(value).expect("20 bytes");
        Ok(GitOID(bytes))
    }
}

impl Debug for GitOID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(&self.0))
    }
}

impl Display for GitOID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(&self.0))
    }
}

pub enum InitTemplate<'a> {
    Dir(Cow<'a, Path>),
    Raw(HashMap<Cow<'a, Path>, Cow<'a, [u8]>>),
}

pub fn pathspec_as_path(pathspec: &str) -> Option<&Path> {
    if !pathspec_is_glob(&pathspec) {
        Some(Path::new(pathspec))
    } else {
        None
    }
}

fn pathspec_is_glob(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'*' | b'?' | b'[' | b']' | b'{' | b'}' => return true,
            b'\\' if i + 1 < bytes.len() => i += 1, // skip 
            _ => {}
        }
        i += 1;
    }

    false
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upstream<'a> {
    remote_name: Cow<'a, str>,
    remote_branch_name: Cow<'a, str>,
}

impl <'a> Upstream<'a> {
    pub fn origin_main() -> Self {
        Upstream {
            remote_name: Cow::Borrowed(ORIGIN),
            remote_branch_name: Cow::Borrowed(MAIN),
        }
    }
    
    pub fn new(remote_name: Cow<'a, str>, remote_branch_name: Cow<'a, str>) -> Self {
        Upstream {
            remote_name,
            remote_branch_name,
        }
    }
    
    pub fn new_owned(remote_name: String, remote_branch_name: String) -> Self {
        Upstream {
            remote_name: Cow::Owned(remote_name),
            remote_branch_name: Cow::Owned(remote_branch_name),
        }
    }
    
    pub fn new_borrowed(remote_name: &'a str, remote_branch_name: &'a str) -> Self {
        Upstream {
            remote_name: Cow::Borrowed(remote_name),
            remote_branch_name: Cow::Borrowed(remote_branch_name),
        }
    }

    pub fn remote_name(&self) -> &str {
        &self.remote_name
    }

    pub fn remote_branch_name(&self) -> &str {
        &self.remote_branch_name
    }
    
    pub fn from_ref_name(ref_name: &'a str) -> Result<Self> {
        // eg, "refs/remotes/origin/main"
        let remote_name_branch = ref_name.strip_prefix("refs/remotes/")
            .ok_or_else(|| Error::State(StateErr::UnsupportedBranchName, ErrSrc::None))?;
            
        let (remote_name, remote_branch_name) = remote_name_branch.split_once('/')
            .ok_or_else(|| Error::State(StateErr::UnsupportedBranchName, ErrSrc::None))?;
        
        Ok(Self::new_borrowed(remote_name, remote_branch_name))
    }
}

pub(crate) struct MergeModeMeta {
    pub(crate) mode: String, 
    pub(crate) head: String, 
    pub(crate) msg: String, 
}

impl MergeModeMeta {
    pub(crate) fn read(top_dir: &Path) -> Result<Self> {
        let dot_git = top_dir.join(DOT_GIT);
        let mode = fs::read_to_string(&dot_git.join(MERGE_MODE))
            .map_err(|e| Error::State(StateErr::InternalsFileIO, ErrSrc::Io(e)))?
            .trim().into();
        let head = fs::read_to_string(&dot_git.join(MERGE_HEAD))
            .map_err(|e| Error::State(StateErr::InternalsFileIO, ErrSrc::Io(e)))?
            .trim().into();
        let msg = fs::read_to_string(&dot_git.join(MERGE_MSG))
            .map_err(|e| Error::State(StateErr::InternalsFileIO, ErrSrc::Io(e)))?
            .trim().into();
        
        Ok(Self { mode, head, msg })
    }
    
    pub(crate) fn write(&self, top_dir: &Path) -> Result<()> {
        let dot_git = top_dir.join(DOT_GIT);
        fs::write(&dot_git.join(MERGE_MSG), &self.msg)
            .map_err(|e| Error::State(StateErr::InternalsFileIO, ErrSrc::Io(e)))?;
        fs::write(&dot_git.join(MERGE_HEAD), &self.head)
            .map_err(|e| Error::State(StateErr::InternalsFileIO, ErrSrc::Io(e)))?;
        fs::write(&dot_git.join(MERGE_MODE), &self.mode)
            .map_err(|e| Error::State(StateErr::InternalsFileIO, ErrSrc::Io(e)))?;
        Ok(())
    }
    
    pub(crate) fn delete(top_dir: &Path) -> Result<()> {
        const FILENAMES: [&'static str; 3] = [
            MERGE_HEAD,
            MERGE_MODE,
            MERGE_MSG
        ];
        
        let dot_git = top_dir.join(DOT_GIT);
        for filename in FILENAMES {
            let filepath = dot_git.join(filename);
            if filepath.exists() {
                fs::remove_file(filepath)
                    .map_err(|e| Error::State(StateErr::InternalsFileIO, ErrSrc::Io(e)))?;
            }
        }
        
        Ok(())
    }
    
    pub(crate) fn exists(top_dir: &Path) -> Result<bool> {
        top_dir.join(DOT_GIT).join(MERGE_HEAD)
            .try_exists()
            .map_err(|e| Error::State(StateErr::InternalsFileIO, ErrSrc::Io(e)))
    }
    
    pub(crate) fn format_msg(source_rev: &str, dest_rev: &str, status: &Status) -> String {
        let conflict_lines = status.changes_iter()
            .filter(|(_,chg)| chg.is_conflicted())
            .map(|(p, _)| ["#      ", &p.to_string_lossy()].concat())
            .collect::<Vec<String>>();
        
        let conflict_lines = if conflict_lines.is_empty() {
            String::new()
        } else {
            ["\n# Conflicts:\n", &conflict_lines.join("\n"), "\n"].concat()
        };
        
        
        format!("Merge branch '{dest_rev}' into {source_rev}\n{conflict_lines}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Resolution {
    Unmodified,
    FastForwarded,
    AutoResolved,
    Unresolved(Status)
}

impl Resolution {
    pub fn is_unmodified(&self) -> bool {
        *self == Self::Unmodified
    }
    
    pub fn is_fast_forwarded(&self) -> bool {
        *self == Self::FastForwarded
    }
    
    pub fn is_auto_resolved(&self) -> bool {
        *self == Self::AutoResolved
    }
    
    pub fn is_unresolved(&self) -> bool {
        matches!(self, Self::Unresolved(_))
    }
    
    pub fn is_committed(&self) -> bool {
        matches!(self, Self::Unmodified | Self::FastForwarded)
    }
}

pub enum SignatureKind {
    GpgSha1,
    GpgSha256,
    Ssh256,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_oids() {
        const HASH_HEX: &'static str = "c4320bb856b4e2547961da6039877478eac076b3";
        assert_eq!(HASH_HEX, GitOID::from_str(HASH_HEX).unwrap().to_string());
    }
}