use std::{collections::HashSet, sync::LazyLock, time::Duration};

use http::Uri;
use reqwest::redirect;
use tower_http::follow_redirect::{self as tower_redirect, policy::PolicyExt};
use url::Url;

use crate::tools::check;

#[derive(Clone)]
pub struct DetectCycle
{
	uris: HashSet<Uri>
}

impl<B, E> tower_redirect::policy::Policy<B, E> for DetectCycle
{
	fn redirect(
		&mut self,
		attempt: &tower_redirect::policy::Attempt<'_>
	) -> Result<tower_redirect::policy::Action, E>
	{
		if self.uris.contains(attempt.location())
		{
			Ok(tower_redirect::policy::Action::Stop)
		}
		else
		{
			self.uris.insert(attempt.previous().clone());
			Ok(tower_redirect::policy::Action::Follow)
		}
	}
}

pub(crate) static REQWEST_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
	check!(
		reqwest::Client::builder()
			.https_only(true)
			.connection_verbose(true)
			.http2_keep_alive_interval(Some(Duration::from_secs(20)))
			.http2_keep_alive_timeout(Duration::from_secs(15))
			.http2_keep_alive_while_idle(true)
			.tls_info(true)
			.http09_responses()
            /* .redirect(redirect::Policy::none())
			.connector_layer(tower_redirect::FollowRedirectLayer::with_policy(
				tower_redirect::policy::Limited::new(15)
					.and(tower_redirect::policy::FilterCredentials::new())
					.and(DetectCycle {
						uris: HashSet::new()
					})
			)) */
			.build()
			.expect("could not build reqwest client")
	)
});
