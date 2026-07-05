let storage = page.local_storage();
storage.set_item("token", "abc123").await?;
let token = storage.get_item("token").await?;
let all = storage.items().await?;
storage.clear().await?;
