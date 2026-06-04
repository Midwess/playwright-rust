page.route("**/*.png", |route| async move {
    route.abort().await
}).await?;
