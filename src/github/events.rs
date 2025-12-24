use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubEvent {
    pub event_type: String,
    pub action: Option<String>,
    pub repository: Repository,
    pub sender: User,
    #[serde(flatten)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub full_name: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseEvent {
    pub action: String,
    pub release: Release,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestEvent {
    pub action: String,
    pub pull_request: PullRequest,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub html_url: String,
    pub merged: Option<bool>,
    pub labels: Vec<Label>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueEvent {
    pub action: String,
    pub issue: Issue,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub html_url: String,
    pub labels: Vec<Label>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRunEvent {
    pub action: String,
    pub workflow_run: WorkflowRun,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: u64,
    pub name: Option<String>,
    pub conclusion: Option<String>,
    pub html_url: String,
}

#[derive(Debug, Clone)]
pub enum ParsedEvent {
    Release(ReleaseEvent),
    PullRequest(PullRequestEvent),
    Issue(IssueEvent),
    WorkflowRun(WorkflowRunEvent),
    Unknown,
}

impl ParsedEvent {
    pub fn from_payload(event_type: &str, payload: &[u8]) -> Result<Self, serde_json::Error> {
        match event_type {
            "release" => Ok(Self::Release(serde_json::from_slice(payload)?)),
            "pull_request" => Ok(Self::PullRequest(serde_json::from_slice(payload)?)),
            "issues" => Ok(Self::Issue(serde_json::from_slice(payload)?)),
            "workflow_run" => Ok(Self::WorkflowRun(serde_json::from_slice(payload)?)),
            _ => Ok(Self::Unknown),
        }
    }

    pub fn event_key(&self) -> Option<String> {
        match self {
            Self::Release(e) => Some(format!("release.{}", e.action)),
            Self::PullRequest(e) => Some(format!("pull_request.{}", e.action)),
            Self::Issue(e) => Some(format!("issues.{}", e.action)),
            Self::WorkflowRun(e) => Some(format!("workflow_run.{}", e.action)),
            Self::Unknown => None,
        }
    }

    pub fn repo_full_name(&self) -> Option<&str> {
        match self {
            Self::Release(e) => Some(&e.repository.full_name),
            Self::PullRequest(e) => Some(&e.repository.full_name),
            Self::Issue(e) => Some(&e.repository.full_name),
            Self::WorkflowRun(e) => Some(&e.repository.full_name),
            Self::Unknown => None,
        }
    }

    pub fn actor(&self) -> Option<&str> {
        match self {
            Self::Release(e) => Some(&e.sender.login),
            Self::PullRequest(e) => Some(&e.sender.login),
            Self::Issue(e) => Some(&e.sender.login),
            Self::WorkflowRun(e) => Some(&e.sender.login),
            Self::Unknown => None,
        }
    }

    pub fn labels(&self) -> Vec<String> {
        match self {
            Self::PullRequest(e) => e
                .pull_request
                .labels
                .iter()
                .map(|l| l.name.clone())
                .collect(),
            Self::Issue(e) => e.issue.labels.iter().map(|l| l.name.clone()).collect(),
            _ => vec![],
        }
    }

    pub fn is_merged(&self) -> bool {
        match self {
            Self::PullRequest(e) => e.pull_request.merged.unwrap_or(false),
            _ => false,
        }
    }
}
