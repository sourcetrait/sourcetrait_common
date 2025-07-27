use crate::*;
use chrono::{DateTime, Utc};
use std::{collections::HashSet, str::Lines, sync::Arc};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GitUser {
    name: String,
    email: String,
}

impl GitUser {
    pub fn new(name: String, email: String) -> Self {
        GitUser { name, email }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }
}

#[derive(Debug, Clone)]
pub struct Log {
    pub(crate) commits: Vec<Commit>,
    pub(crate) users: HashSet<Arc<GitUser>>,
    pub(crate) signature_fingerprints: HashSet<Arc<String>>,
}

impl Log {
    pub fn commits(&self) -> &Vec<Commit> {
        &self.commits
    }

    pub fn users(&self) -> &HashSet<Arc<GitUser>> {
        &self.users
    }

    pub fn signature_fingerprints(&self) -> &HashSet<Arc<String>> {
        &self.signature_fingerprints
    }
}

impl Log {
    // todo: this does not belong here. move to cli.rs
    pub(crate) fn from_cli(s: &str, options: Option<LogOptions>) -> Result<Self> {
        let mut commits = Vec::new();
        let mut users: HashSet<Arc<GitUser>> = HashSet::new();
        let mut signature_fingerprints: HashSet<Arc<String>> = HashSet::new();

        let (show_message, show_signature_fingerprint) = if let Some(options) = &options {
            (options.show_message, options.show_signature_fingerprint)
        } else {
            (false, false)
        };

        let mut lines = s.lines();
        while let Some(line) = lines.next() {
            let commit_oid = GitOID::from_str(line.trim())?;

            let tree_oid = lines.next()
                .ok_or_else(|| Error::GitLogParse)
                .map(|s| GitOID::from_str(s.trim()))??;

            let parent_oids = lines.next()
                .ok_or_else(|| Error::GitLogParse)
                .map(|s| s.split_whitespace())?
                .map(GitOID::from_str)
                .collect::<Result<Vec<_>>>()?;


            let (author, author_time) = Self::parse_user(&mut lines)?;
            let (committer, committer_time) = Self::parse_user(&mut lines)?;

            let author = if let Some(user) = users.get(&author) {
                user.clone()
            } else {
                users.insert(author.clone());
                author
            };

            let committer = if let Some(user) = users.get(&committer) {
                user.clone()
            } else {
                users.insert(committer.clone());
                committer
            };

            let message = if show_message {
                let mut msg: Vec<&str> = Vec::new();
                while let Some(line) = lines.next() && !line.starts_with("::EOF") {
                    msg.push(line.trim());
                }

                Some(msg.join("\n").trim().into())
            } else {
                None
            };

            let signature_fingerprint = if show_signature_fingerprint && let Some(line) = lines.next() {
                let line = line.trim();
                if !line.is_empty() {
                    let fingerprint = Arc::new(line.trim().into());

                    if let Some(fingerprint) = signature_fingerprints.get(&fingerprint) {
                        Some(fingerprint.clone())
                    } else {
                        signature_fingerprints.insert(fingerprint.clone());
                        Some(fingerprint)
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let commit = Commit {
                commit_oid,
                tree_oid,
                parent_oid: parent_oids,
                author,
                author_time,
                committer,
                committer_time,
                signature_fingerprint,
                message,
            };

            commits.push(commit);
        }


        Ok(Log {
            commits,
            users,
            signature_fingerprints,
        })
    }

    fn parse_user(lines: &mut Lines<'_>) -> Result<(Arc<GitUser>, DateTime<Utc>)> {
        let name = lines.next()
            .ok_or_else(|| Error::GitLogParse)?
            .into();
        let email = lines.next()
            .ok_or_else(|| Error::GitLogParse)?
            .into();
        let time = lines.next()
            .ok_or_else(|| Error::GitLogParse)
            .map(|s| s.parse::<i64>())?
            .map(|n| DateTime::<Utc>::from_timestamp(n, 0))
            .map_err(|_| Error::GitLogParse)?
            .ok_or_else(|| Error::GitLogParse)?;

        let user = Arc::new(GitUser { name, email });
        Ok((user, time))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub commit_oid: GitOID,
    pub tree_oid: GitOID,
    pub parent_oid: Vec<GitOID>,
    pub author: Arc<GitUser>,
    pub author_time: chrono::DateTime<chrono::Utc>,
    pub committer: Arc<GitUser>,
    pub committer_time: chrono::DateTime<chrono::Utc>,
    pub signature_fingerprint: Option<Arc<String>>,
    pub message: Option<String>,
}

impl Commit {
    pub fn commit_oid(&self) -> &GitOID {
        &self.commit_oid
    }

    pub fn tree_oid(&self) -> &GitOID {
        &self.tree_oid
    }

    pub fn parent_oids(&self) -> &Vec<GitOID> {
        &self.parent_oid
    }

    pub fn author(&self) -> &GitUser {
        &self.author
    }

    pub fn author_time(&self) -> chrono::DateTime<chrono::Utc> {
        self.author_time
    }

    pub fn committer(&self) -> &GitUser {
        &self.committer
    }

    pub fn committer_time(&self) -> chrono::DateTime<chrono::Utc> {
        self.committer_time
    }

    pub fn signature_fingerprint(&self) -> Option<Arc<String>> {
        self.signature_fingerprint.as_ref().map(|f| f.clone())
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

pub(crate) const LOG_FORMAT: &'static str = "--format=format:%H%n%T%n%P%n%an%n%ae%n%at%n%cn%n%ce%n%ct";
pub(crate) const LOG_FORMAT_MESSAGE: &'static str = "%n%B::EOF";
pub(crate) const LOG_FORMAT_SIGNATURE_FINGERPRINT: &'static str = "%n%GF";

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, str::FromStr, sync::Arc};

    #[test]
    fn test_log() {
        // Example test for GitLog
        let user = Arc::new(super::GitUser {
            name: "Alice".to_string(),
            email: "alice@wonderland.email".to_string(),
        });

        let sig = Arc::new(
            "-----BEGIN SSH SIGNATURE-----
         U1NIU0lHAAAAAQAAADMAAAALc3OoLWVkMjU1MTkAAAAgFt1UqLW9/7g4bm5fkid8e6wzmN
         bfaqtJ/NBe4v7SOFQAAAADZ2l0ANAAAAAAAAZzaGE1MTIAAABTAAAAC3NzaC1lZDI1NTE5
         AAAAQDEnI973DjCjjPC6GXqYBozuy64JgkUqlJhtgxokbCj2PzoXTSDiVvA3LvlrYeeI6m
         iCx5nQsuNIA2VHfG5q4wk=
         -----END SSH SIGNATURE-----"
                .to_string(),
        );

        let commit1 = super::Commit {
            commit_oid: super::GitOID::from_str("c4320bb856b4e2547961da6039877478eac076b3")
                .unwrap(),
            tree_oid: super::GitOID::from_str("cb1df81c00a3a058d25ecd665e03ce682901ab71")
                .unwrap(),
            parent_oid: vec![
                super::GitOID::from_str("b0b81f3e1047e413f0ff2764197b4cdac47684bf").unwrap(),
            ],
            author: user.clone(),
            author_time: chrono::DateTime::from_timestamp_millis(1751894159).unwrap(),
            committer: user.clone(),
            committer_time: chrono::DateTime::from_timestamp_millis(1751894159).unwrap(),
            signature_fingerprint: Some(sig.clone()),
            message: Some("iface uses _with fns for options".to_string()),
        };

        let commit2 = super::Commit {
            commit_oid: super::GitOID::from_str("c4320bb856b4e2547961da6039877478eac076b3")
                .unwrap(),
            tree_oid: super::GitOID::from_str("cb1df81c00a3a058d25ecd665e03ce682901ab71")
                .unwrap(),
            parent_oid: vec![
                super::GitOID::from_str("b0b81f3e1047e413f0ff2764197b4cdac47684bf").unwrap(),
            ],
            author: user.clone(),
            author_time: chrono::DateTime::from_timestamp_millis(1751894159).unwrap(),
            committer: user.clone(),
            committer_time: chrono::DateTime::from_timestamp_millis(1751894159).unwrap(),
            signature_fingerprint: Some(sig.clone()),
            message: Some("iface uses _with fns for options".to_string()),
        };

        let log = super::Log {
            commits: vec![commit1, commit2],
            users: HashSet::from([user]),
            signature_fingerprints: HashSet::from([sig]),
        };

        assert_eq!(
            "c4320bb856b4e2547961da6039877478eac076b3",
            log.commits[0].commit_oid.to_string()
        );
    }
}
