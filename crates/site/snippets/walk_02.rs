page.locator("[data-lang='Java']").await.click(None).await?;

expect(page.locator("[data-lang='Java']").await)
    .to_have_attribute("aria-selected", "true")
    .await?;
expect(page.locator("#comparison").await)
    .to_contain_text("Playwright.create()")
    .await?;
