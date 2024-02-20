use octocrab::models::pulls::PullRequest as OctocrabPR;

#[derive(Debug)]
pub struct PullRequest {
    pub url: String,
    pub id: u64,
    pub number: u64,
    pub title: Option<String>,
    pub user: Option<String>,
    pub body: Option<String>,
    pub labels: Option<Vec<String>>,
    pub milestone: Option<String>,
    pub assignees: Option<Vec<String>>,
    pub base: String,
    pub draft: Option<bool>,
}

impl PullRequest {
    pub fn new(
        url: String,
        id: u64,
        number: u64,
        title: Option<String>,
        user: Option<String>,
        body: Option<String>,
        labels: Option<Vec<String>>,
        milestone: Option<String>,
        assignees: Option<Vec<String>>,
        base: String,
        draft: Option<bool>,
    ) -> Self {
        Self {
            url,
            id,
            number,
            title,
            user,
            body,
            labels,
            milestone,
            assignees,
            base,
            draft,
        }
    }
}

impl From<OctocrabPR> for PullRequest {
    fn from(value: OctocrabPR) -> Self {
        Self {
            url: value.url,
            id: *value.id,
            number: value.number,
            title: value.title,
            user: value.user.map(|u| u.login),
            body: value.body,
            labels: value
                .labels
                .map(|l| l.into_iter().map(|l| l.name).collect()),
            milestone: value.milestone.map(|m| m.title),
            assignees: value
                .assignees
                .map(|a| a.into_iter().map(|a| a.login).collect()),
            base: value.base.ref_field.to_string(),
            draft: value.draft,
        }
    }
}
