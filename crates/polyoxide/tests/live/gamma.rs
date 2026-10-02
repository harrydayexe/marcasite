//! Gamma API live tests.

use futures_util::{StreamExt as _, TryStreamExt as _};
use polyoxide::gamma::{Market, Tag};

use crate::common::{GAMMA, check, get, pm, sample};

#[tokio::test]
#[ignore = "live network"]
async fn get_status() {
    let status = pm().gamma().get_status().await.unwrap();
    assert!(!status.is_empty());
}

#[tokio::test]
#[ignore = "live network"]
async fn list_tags() {
    let tags = pm().gamma().list_tags().limit(5).send().await.unwrap();
    assert!(!tags.is_empty());
    check::<Vec<Tag>>("GET /tags", &get(GAMMA, "/tags", &[("limit", "20")]).await);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_tags_stream_pages() {
    let tags: Vec<Tag> = pm()
        .gamma()
        .list_tags()
        .limit(3)
        .into_stream()
        .take(7)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(tags.len(), 7);
}

#[tokio::test]
#[ignore = "live network"]
async fn get_tag_by_id_and_slug() {
    let s = sample().await;
    let by_id = pm()
        .gamma()
        .get_tag(s.tag_id.as_str())
        .send()
        .await
        .unwrap();
    let by_slug = pm()
        .gamma()
        .get_tag_by_slug(&s.tag_slug)
        .send()
        .await
        .unwrap();
    assert_eq!(by_id.id, by_slug.id);
    check::<Tag>(
        "GET /tags/{id}",
        &get(GAMMA, &format!("/tags/{}", s.tag_id), &[]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_market() {
    let s = sample().await;
    let market = pm()
        .gamma()
        .get_market(s.market_id.as_str())
        .send()
        .await
        .unwrap();
    assert_eq!(market.id.unwrap().as_str(), s.market_id);
    check::<Market>(
        "GET /markets/{id}",
        &get(GAMMA, &format!("/markets/{}", s.market_id), &[]).await,
    );
}
