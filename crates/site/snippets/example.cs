using Microsoft.Playwright;

using var playwright = await Playwright.CreateAsync();
var browser = await playwright.Chromium.LaunchAsync();
var page = await browser.NewPageAsync();
await page.GotoAsync("https://example.com");

var heading = page.Locator("h1");
Assert.AreEqual("Example Domain", await heading.TextContentAsync());

await browser.CloseAsync();
