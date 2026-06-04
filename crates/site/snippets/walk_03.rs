page.locator("#feature-cross-browser [data-lang='Firefox']")
    .await
    .click(None)
    .await?;

expect(page.locator("#feature-cross-browser").await)
    .to_contain_text("firefox")
    .await?;
