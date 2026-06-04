from playwright.sync_api import sync_playwright

with sync_playwright() as p:
    browser = p.chromium.launch()
    page = browser.new_page()
    page.goto("https://example.com")

    heading = page.locator("h1")
    assert heading.text_content() == "Example Domain"

    browser.close()
