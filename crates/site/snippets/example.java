import com.microsoft.playwright.*;

try (Playwright playwright = Playwright.create()) {
    Browser browser = playwright.chromium().launch();
    Page page = browser.newPage();
    page.navigate("https://example.com");

    Locator heading = page.locator("h1");
    assertEquals("Example Domain", heading.textContent());

    browser.close();
}
