tracing_subscriber::fmt().init();
page.goto(url, None).await?; // emits a span
