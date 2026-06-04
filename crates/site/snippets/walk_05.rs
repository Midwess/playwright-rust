expect(page.locator("#disclaimer").await)
    .to_contain_text("unofficial")
    .await?;
