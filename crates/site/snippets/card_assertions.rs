expect(page.locator("#status").await)
    .to_have_text("Ready")
    .await?;
