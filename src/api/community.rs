use crate::{
    api::{get, NameOrId},
    pagination::PAGE_ITEMS,
};
use anyhow::Error;
use lemmy_api_common::{
    community::{GetCommunity, GetCommunityResponse, ListCommunities, ListCommunitiesResponse},
    sensitive::Sensitive,
};
use lemmy_db_schema::{newtypes::CommunityId, ListingType, SortType};

pub async fn list_communities(
    page: Option<i32>,
    auth: Option<Sensitive<String>>,
) -> Result<ListCommunitiesResponse, Error> {
    let params = ListCommunities {
        type_: Some(ListingType::All),
        sort: Some(SortType::TopMonth),
        page: page.map(Into::into),
        limit: Some(PAGE_ITEMS.into()),
        auth,
    };
    get("/community/list", params).await
}

pub async fn get_community(
    name_or_id: NameOrId,
    auth: Option<Sensitive<String>>,
) -> Result<GetCommunityResponse, Error> {
    let mut params = GetCommunity {
        auth,
        ..Default::default()
    };
    match name_or_id {
        NameOrId::Name(n) => params.name = Some(n),
        NameOrId::Id(c) => params.id = Some(CommunityId(c)),
    }
    get("/community", params).await
}