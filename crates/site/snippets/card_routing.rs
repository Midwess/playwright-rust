page.route("**/*.png", |route| async move {
    route.abort(None).await
}).await?;
